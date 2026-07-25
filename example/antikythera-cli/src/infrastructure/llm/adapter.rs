use antikythera_core::domain::types::{ChatMessage, MessagePart};
use serde_json::{Value, json};

/// Adapter for converting core `ChatMessage` instances to provider wire formats.
pub struct MessageAdapter;

impl MessageAdapter {
    fn part_to_gemini(part: &MessagePart) -> Value {
        match part {
            MessagePart::Text { text } => json!({"text": text}),
            MessagePart::Image { mime_type, data } => json!({
                "inline_data": { "mime_type": mime_type, "data": data }
            }),
            MessagePart::File {
                mime_type, data, ..
            } => json!({
                "inline_data": { "mime_type": mime_type, "data": data }
            }),
        }
    }

    fn part_to_openai(part: &MessagePart) -> Value {
        match part {
            MessagePart::Text { text } => json!({"type": "text", "text": text}),
            MessagePart::Image { mime_type, data } => json!({
                "type": "image_url",
                "image_url": { "url": format!("data:{};base64,{}", mime_type, data) }
            }),
            MessagePart::File {
                mime_type, data, ..
            } => json!({
                "type": "image_url",
                "image_url": { "url": format!("data:{};base64,{}", mime_type, data) }
            }),
        }
    }

    /// Convert messages to OpenAI-compatible format.
    ///
    /// Returns `[{"role": "...", "content": "..."}]`.
    /// Multi-part messages are returned as an array of content objects.
    pub fn to_openai_format(messages: &[ChatMessage]) -> Vec<Value> {
        messages
            .iter()
            .map(|msg| {
                let all_text = msg
                    .parts
                    .iter()
                    .all(|p| matches!(p, MessagePart::Text { .. }));

                if all_text {
                    json!({"role": msg.role.as_str(), "content": msg.content()})
                } else {
                    json!({
                        "role": msg.role.as_str(),
                        "content": msg.parts.iter().map(Self::part_to_openai).collect::<Vec<_>>()
                    })
                }
            })
            .collect()
    }

    /// Convert messages to Ollama format (simplified OpenAI-like structure).
    pub fn to_ollama_format(messages: &[ChatMessage]) -> Vec<Value> {
        messages
            .iter()
            .map(|msg| json!({"role": msg.role.as_str(), "content": msg.content()}))
            .collect()
    }

    /// Convert messages to Gemini format.
    ///
    /// Returns `(system_instruction_text, contents)`.
    /// System messages are extracted into the first return value; all other
    /// messages are placed in `contents` with `"user"` / `"model"` roles.
    pub fn to_gemini_format(messages: &[ChatMessage]) -> (Option<String>, Vec<Value>) {
        let mut system_parts = Vec::new();
        let mut contents = Vec::new();

        for message in messages {
            match message.role.as_str() {
                "system" => system_parts.push(message.content()),
                "user" => {
                    let parts: Vec<Value> =
                        message.parts.iter().map(Self::part_to_gemini).collect();
                    contents.push(json!({"role": "user", "parts": parts}));
                }
                "assistant" => {
                    let parts: Vec<Value> =
                        message.parts.iter().map(Self::part_to_gemini).collect();
                    contents.push(json!({"role": "model", "parts": parts}));
                }
                _ => {}
            }
        }

        let system_instruction = if system_parts.is_empty() {
            None
        } else {
            Some(system_parts.join("\n\n"))
        };

        (system_instruction, contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use antikythera_core::domain::types::{ChatMessage, MessagePart, MessageRole};

    #[test]
    fn test_to_openai_format_text_only() {
        let messages = vec![
            ChatMessage::new(MessageRole::User, "hello"),
            ChatMessage::new(MessageRole::Assistant, "hi there"),
        ];
        let result = MessageAdapter::to_openai_format(&messages);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["role"], "user");
        assert_eq!(result[0]["content"], "hello");
        assert_eq!(result[1]["role"], "assistant");
        assert_eq!(result[1]["content"], "hi there");
    }

    #[test]
    fn test_to_openai_format_with_image() {
        let messages = vec![ChatMessage::with_parts(
            MessageRole::User,
            vec![
                MessagePart::text("look at this"),
                MessagePart::image("image/png", "base64data"),
            ],
        )];
        let result = MessageAdapter::to_openai_format(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "user");
        let content = &result[0]["content"];
        assert!(content.is_array());
        let arr = content.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["type"], "text");
        assert_eq!(arr[0]["text"], "look at this");
        assert_eq!(arr[1]["type"], "image_url");
        assert!(arr[1]["image_url"]["url"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,"));
    }

    #[test]
    fn test_to_gemini_format_text_only() {
        let messages = vec![
            ChatMessage::new(MessageRole::System, "You are helpful."),
            ChatMessage::new(MessageRole::User, "hi"),
            ChatMessage::new(MessageRole::Assistant, "hello!"),
        ];
        let (system, contents) = MessageAdapter::to_gemini_format(&messages);
        assert_eq!(system.as_deref(), Some("You are helpful."));
        assert_eq!(contents.len(), 2);
        assert_eq!(contents[0]["role"], "user");
        assert_eq!(contents[0]["parts"][0]["text"], "hi");
        assert_eq!(contents[1]["role"], "model");
        assert_eq!(contents[1]["parts"][0]["text"], "hello!");
    }

    #[test]
    fn test_to_gemini_format_no_system() {
        let messages = vec![ChatMessage::new(MessageRole::User, "test")];
        let (system, contents) = MessageAdapter::to_gemini_format(&messages);
        assert!(system.is_none());
        assert_eq!(contents.len(), 1);
    }

    #[test]
    fn test_to_ollama_format_text_only() {
        let messages = vec![
            ChatMessage::new(MessageRole::User, "question"),
            ChatMessage::new(MessageRole::Assistant, "answer"),
        ];
        let result = MessageAdapter::to_ollama_format(&messages);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["role"], "user");
        assert_eq!(result[0]["content"], "question");
        assert_eq!(result[1]["role"], "assistant");
        assert_eq!(result[1]["content"], "answer");
    }

    #[test]
    fn test_to_ollama_format_empty() {
        let messages: Vec<ChatMessage> = vec![];
        let result = MessageAdapter::to_ollama_format(&messages);
        assert!(result.is_empty());
    }
}
