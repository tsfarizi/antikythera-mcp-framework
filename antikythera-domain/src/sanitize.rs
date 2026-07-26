//! Text sanitization utilities for TOML-safe strings.

/// Characters that are problematic in TOML basic strings.
const TOML_UNSAFE_CHARS: &[char] = &['\n', '\r', '\t', '\\', '"'];

/// Sanitize a string to be TOML-safe for use in basic strings.
///
/// This function:
/// - Removes newlines and replaces with spaces
/// - Removes emojis and special Unicode characters
/// - Escapes backslashes and quotes
/// - Trims excess whitespace
pub fn sanitize_for_toml(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_was_space = false;

    for c in input.chars() {
        match c {
            // Replace newlines/tabs with space
            '\n' | '\r' | '\t' => {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            }
            // Escape backslashes
            '\\' => {
                result.push_str("\\\\");
                prev_was_space = false;
            }
            // Escape quotes
            '"' => {
                result.push_str("\\\"");
                prev_was_space = false;
            }
            // Keep ASCII printable characters
            c if c.is_ascii() && !c.is_ascii_control() => {
                if c == ' ' {
                    if !prev_was_space {
                        result.push(c);
                        prev_was_space = true;
                    }
                } else {
                    result.push(c);
                    prev_was_space = false;
                }
            }
            // Keep common non-ASCII letters (Indonesian uses standard Latin)
            c if c.is_alphabetic() => {
                result.push(c);
                prev_was_space = false;
            }
            // Remove emojis and other special characters, replace with space
            _ => {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            }
        }
    }

    result.trim().to_string()
}

/// Check if a string contains characters that would break TOML basic string parsing.
pub fn needs_sanitization(input: &str) -> bool {
    input
        .chars()
        .any(|c| TOML_UNSAFE_CHARS.contains(&c) || c.is_control() || is_emoji(c))
}

/// Check if a character is likely an emoji.
fn is_emoji(c: char) -> bool {
    let code = c as u32;
    // Common emoji ranges (non-overlapping)
    matches!(code,
        0x1F300..=0x1F9FF |  // Misc Symbols, Pictographs, Emoticons, Transport, etc.
        0x2600..=0x26FF |    // Misc symbols
        0x2700..=0x27BF |    // Dingbats
        0xFE00..=0xFE0F |    // Variation Selectors
        0x200D                // Zero Width Joiner
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_string_unchanged() {
        assert_eq!(sanitize_for_toml("hello world"), "hello world");
    }

    #[test]
    fn newlines_replaced_with_space() {
        assert_eq!(sanitize_for_toml("hello\nworld"), "hello world");
        assert_eq!(sanitize_for_toml("a\r\nb"), "a b");
    }

    #[test]
    fn backslashes_escaped() {
        assert_eq!(sanitize_for_toml("a\\b"), "a\\\\b");
    }

    #[test]
    fn quotes_escaped() {
        assert_eq!(sanitize_for_toml(r#"say "hi""#), r#"say \"hi\""#);
    }

    #[test]
    fn needs_sanitization_detects_unsafe_chars() {
        assert!(needs_sanitization("has\nnewline"));
        assert!(needs_sanitization("has\"quote"));
        assert!(needs_sanitization("has\\backslash"));
        assert!(!needs_sanitization("safe string"));
    }
}
