//! URL validation with configurable allowed/blocked regex patterns.

use regex::Regex;

use super::types::ValidationResult;

/// Regex-based URL validator.
pub struct URLValidator {
    allowed_patterns: Vec<Regex>,
    blocked_patterns: Vec<Regex>,
}

impl URLValidator {
    pub fn new(allowed: Vec<Regex>, blocked: Vec<Regex>) -> Self {
        Self {
            allowed_patterns: allowed,
            blocked_patterns: blocked,
        }
    }

    /// Validate a URL against blocked then allowed patterns.
    pub fn validate(&self, url: &str) -> ValidationResult {
        for regex in &self.blocked_patterns {
            if regex.is_match(url) {
                return ValidationResult::Invalid(format!("URL matches blocked pattern: {url}"));
            }
        }

        let is_allowed = self
            .allowed_patterns
            .iter()
            .any(|regex| regex.is_match(url));

        if !is_allowed {
            return ValidationResult::Invalid(format!(
                "URL does not match any allowed pattern: {url}"
            ));
        }

        ValidationResult::Valid
    }
}
