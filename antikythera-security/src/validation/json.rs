//! JSON structure validation (nesting depth, array length).

use serde_json::Value;

/// Validates JSON structure against depth and array-length limits.
pub struct JSONValidator {
    max_nesting_depth: u32,
    max_array_length: u32,
}

impl JSONValidator {
    pub fn new(max_depth: u32, max_array_len: u32) -> Self {
        Self {
            max_nesting_depth: max_depth,
            max_array_length: max_array_len,
        }
    }

    /// Recursively validate a `serde_json::Value` tree.
    pub fn validate_structure(&self, value: &Value, depth: u32) -> Result<(), String> {
        if depth > self.max_nesting_depth {
            return Err(format!(
                "JSON nesting depth {depth} exceeds maximum {}",
                self.max_nesting_depth
            ));
        }

        match value {
            Value::Array(arr) => {
                if arr.len() as u32 > self.max_array_length {
                    return Err(format!(
                        "JSON array length {} exceeds maximum {}",
                        arr.len(),
                        self.max_array_length
                    ));
                }
                for item in arr {
                    self.validate_structure(item, depth + 1)?;
                }
            }
            Value::Object(obj) => {
                for v in obj.values() {
                    self.validate_structure(v, depth + 1)?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
