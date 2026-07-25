//! Security implementations (CLI-owned).
//!
//! Concrete implementations of security port traits defined in core.

pub mod rate_limit;
pub mod secrets;
pub mod validation;

// Re-export for convenience
pub use rate_limit::RateLimiter;
pub use secrets::SecretManager;
pub use validation::{InputValidator, InputValidatorError, ValidationResult, ValidationError};
