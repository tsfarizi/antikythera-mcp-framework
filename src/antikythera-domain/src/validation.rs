//! Domain validation rules.
//!
//! Business rules that validate inputs against the MCP specification.

/// Validates a tool name against the MCP spec naming rules:
/// - Length 1-128 characters
/// - Allowed characters: A-Z, a-z, 0-9, underscore (_), hyphen (-), dot (.)
/// - No spaces, commas, or other special characters
pub fn validate_tool_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("tool name must not be empty".to_string());
    }
    if name.len() > 128 {
        return Err(format!(
            "tool name exceeds 128 characters (length: {})",
            name.len()
        ));
    }
    for ch in name.chars() {
        if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' && ch != '.' {
            return Err(format!(
                "tool name contains invalid character '{}'. Allowed: A-Z, a-z, 0-9, underscore, hyphen, dot",
                ch
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_tool_names_accepted() {
        assert!(validate_tool_name("my_tool").is_ok());
        assert!(validate_tool_name("tool-name").is_ok());
        assert!(validate_tool_name("tool.name").is_ok());
        assert!(validate_tool_name("a").is_ok());
        assert!(validate_tool_name("search123").is_ok());
    }

    #[test]
    fn empty_name_rejected() {
        assert!(validate_tool_name("").is_err());
    }

    #[test]
    fn too_long_name_rejected() {
        let long = "a".repeat(129);
        assert!(validate_tool_name(&long).is_err());
    }

    #[test]
    fn invalid_characters_rejected() {
        assert!(validate_tool_name("has space").is_err());
        assert!(validate_tool_name("has,comma").is_err());
        assert!(validate_tool_name("has@at").is_err());
    }

    #[test]
    fn exactly_128_chars_accepted() {
        let name = "a".repeat(128);
        assert!(validate_tool_name(&name).is_ok());
    }
}
