//! Domain message types — the canonical definitions live in `message_types`.
//! This module re-exports them for backward compatibility.

pub use super::message_types::{Message, MessagePart, MessageRole};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub parts: Vec<MessagePart>,
}

impl ChatMessage {
    /// Create a new text-only message (backwards compatible)
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            parts: vec![MessagePart::text(content)],
        }
    }

    /// Create a message with multiple parts
    pub fn with_parts(role: MessageRole, parts: Vec<MessagePart>) -> Self {
        Self { role, parts }
    }

    /// Get the text content of the message (concatenated from all text parts)
    pub fn content(&self) -> String {
        self.parts
            .iter()
            .filter_map(|p| p.as_text())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Check if this message contains any non-text parts
    pub fn has_attachments(&self) -> bool {
        self.parts
            .iter()
            .any(|p| !matches!(p, MessagePart::Text { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_message_new_text_only() {
        let msg = ChatMessage::new(MessageRole::User, "hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content(), "hello");
        assert!(!msg.has_attachments());
    }

    #[test]
    fn chat_message_with_text_parts_concatenated() {
        let msg = ChatMessage::with_parts(
            MessageRole::Assistant,
            vec![MessagePart::text("a"), MessagePart::text("b")],
        );
        assert_eq!(msg.content(), "ab");
    }

    #[test]
    fn chat_message_has_attachments_with_image() {
        let msg = ChatMessage::with_parts(
            MessageRole::User,
            vec![
                MessagePart::text("see"),
                MessagePart::image("image/png", "base64data"),
            ],
        );
        assert!(msg.has_attachments());
    }

    #[test]
    fn chat_message_serialization_roundtrip() {
        let msg = ChatMessage::new(MessageRole::User, "test");
        let json = serde_json::to_string(&msg).unwrap();
        let restored: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.role, MessageRole::User);
        assert_eq!(restored.content(), "test");
    }
}
