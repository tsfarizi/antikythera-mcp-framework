use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// Canonical MCP tool call envelope used by the runtime boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEnvelope {
    pub tool: String,
    #[serde(default)]
    pub arguments: Value,
    #[serde(default)]
    pub correlation_id: Option<String>,
}

/// Canonical MCP tool result envelope used by the runtime boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultEnvelope {
    pub tool: String,
    pub success: bool,
    #[serde(default)]
    pub output: Value,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EnvelopeError {
    #[error("tool must be a non-empty string")]
    MissingTool,
    #[error("arguments must be a JSON object")]
    InvalidArguments,
    #[error("error must be present for failed results and absent for successful ones")]
    InconsistentResult,
}

impl EnvelopeError {
    /// Build a consistent transport-layer error message.
    pub fn to_transport_message(&self, phase: &str) -> String {
        format!("invalid MCP tool {phase} envelope: {self}")
    }
}

/// Validate strict tool-call envelope contract.
pub fn validate_tool_call_envelope(env: &ToolCallEnvelope) -> Result<(), EnvelopeError> {
    if env.tool.trim().is_empty() {
        return Err(EnvelopeError::MissingTool);
    }
    if !env.arguments.is_object() {
        return Err(EnvelopeError::InvalidArguments);
    }
    Ok(())
}

/// Validate strict tool-result envelope contract.
pub fn validate_tool_result_envelope(env: &ToolResultEnvelope) -> Result<(), EnvelopeError> {
    if env.tool.trim().is_empty() {
        return Err(EnvelopeError::MissingTool);
    }

    match (env.success, env.error.as_ref().map(|s| s.trim().is_empty())) {
        (true, Some(false)) => Err(EnvelopeError::InconsistentResult),
        (false, None) | (false, Some(true)) => Err(EnvelopeError::InconsistentResult),
        _ => Ok(()),
    }
}
