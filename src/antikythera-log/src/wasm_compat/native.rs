//! Native/WASI time implementation — delegates to chrono.

/// Current Unix timestamp in milliseconds.
pub fn now_unix_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// Current time as RFC 3339 string.
pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Current Unix timestamp in nanoseconds.
///
/// Falls back to microseconds * 1000 if nanosecond precision is unavailable.
pub fn now_timestamp_nanos() -> i64 {
    chrono::Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or_else(|| chrono::Utc::now().timestamp_micros() * 1_000)
}
