use crate::policy::RetryPolicy;
use std::future::Future;

/// Execute `f` up to `policy.max_attempts` times, retrying only when
/// `should_retry` returns `true` for the error.
///
/// Back-off sleep is driven by `tokio::time::sleep`.
pub async fn with_retry_if<F, Fut, T, E, SR>(
    policy: &RetryPolicy,
    mut f: F,
    should_retry: SR,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
    SR: Fn(&E) -> bool,
{
    let mut attempt = 0u32;
    loop {
        match f().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                attempt += 1;
                if attempt >= policy.max_attempts || !should_retry(&err) {
                    return Err(err);
                }
                let delay = policy.delay_for_attempt(attempt - 1);
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Execute `f` up to `policy.max_attempts` times, retrying on *every* error.
pub async fn with_retry<F, Fut, T, E>(policy: &RetryPolicy, f: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    with_retry_if(policy, f, |_| true).await
}
