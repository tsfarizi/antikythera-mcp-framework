//! Security error types.

use thiserror::Error;

/// Unified error type for all security subsystems.
#[derive(Debug, Clone, Error)]
pub enum SecurityError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Secret error: {0}")]
    Secret(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
