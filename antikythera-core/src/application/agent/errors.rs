use super::memory::MemoryError;
use crate::application::client::McpError;
use crate::application::tooling::ToolInvokeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error(transparent)]
    Client(#[from] McpError),
    #[error(transparent)]
    Tool(#[from] ToolError),
    #[error("invalid agent response: {0}")]
    InvalidResponse(String),
    #[error("maximum steps exceeded")]
    MaxStepsExceeded,
    #[error("operation timed out")]
    Timeout,
    #[error("memory error: {0}")]
    MemoryError(#[from] MemoryError),
    #[error("rate limit exceeded for session")]
    RateLimited,
}

impl AgentError {
    pub fn user_message(&self) -> String {
        match self {
            AgentError::Client(err) => err.user_message(),
            AgentError::Tool(err) => err.user_message(),
            AgentError::InvalidResponse(msg) => {
                format!(
                    "AI gave an incomprehensible response. Please try rephrasing your instructions. Error: {}",
                    msg
                )
            }
            AgentError::MaxStepsExceeded => {
                "Maximum steps exceeded. Process has been stopped.".to_string()
            }
            AgentError::Timeout => "Operation timed out. Please try again.".to_string(),
            AgentError::MemoryError(err) => {
                format!("State storage error: {}", err)
            }
            AgentError::RateLimited => {
                "Rate limit exceeded. Please wait a moment before trying again.".to_string()
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("unknown tool requested: {0}")]
    UnknownTool(String),
    #[error("tool '{0}' is not bound to any MCP server")]
    UnboundTool(String),
    #[error("failed to execute tool '{tool}': {source}")]
    Execution {
        tool: String,
        #[source]
        source: ToolInvokeError,
    },
}

impl ToolError {
    pub fn user_message(&self) -> String {
        match self {
            ToolError::UnknownTool(name) => {
                format!("Tool \"{name}\" is not available on the server.")
            }
            ToolError::UnboundTool(name) => {
                format!(
                    "Tool \"{name}\" is not bound to any MCP server. Please check the client configuration."
                )
            }
            ToolError::Execution { tool, source } => {
                format!(
                    "Tool \"{tool}\" execution failed: {message}",
                    message = source
                )
            }
        }
    }
}
