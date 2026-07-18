use serde::{Deserialize, Serialize};
pub use crate::domain::types::MessageRole;
use super::dynamic_value::DynamicValue;

/// Chat message in conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: MessageRole::User, content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: MessageRole::Assistant, content: content.into() }
    }
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: MessageRole::System, content: content.into() }
    }
    pub fn tool(content: impl Into<String>) -> Self {
        Self { role: MessageRole::ToolResult, content: content.into() }
    }
}

/// Tool call from LLM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub name: String,
    pub arguments: DynamicValue,
}

/// Tool execution result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    pub name: String,
    pub success: bool,
    pub output: DynamicValue,
    pub error: Option<String>,
}

/// Agent action determined from model response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentAction {
    CallTool(ToolCall),
    FinalResponse(String),
    Error(String),
}
