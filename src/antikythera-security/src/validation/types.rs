//! Validation result types.

use serde::{Deserialize, Serialize};

/// Result of a validation check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
    Sanitized(String),
}

/// A single validation error with field context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}
