//! Wire shapes for the Runtime Bridge protocol.
//!
//! Every shape is serialized exactly as the golden contract
//! (`contracts/shared/wire_protocol.golden.json`) spells it. Field names are
//! per-shape: `llm-request`/`llm-response` are snake_case on the wire, the
//! tool shapes are kebab-case, the envelope mixes `type` with snake_case
//! correlation/session/client ids. No field may be added beyond the golden
//! file (acceptance invariant 5).

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// llm-request / llm-response (wire names are snake_case, matching the golden)
// ---------------------------------------------------------------------------

/// `llm-request` vocabulary record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub messages_json: String,
    pub force_json: bool,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub schema_name: Option<String>,
    pub metadata_json: Option<String>,
}

/// `llm-response` vocabulary record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub message_json: Option<String>,
    pub tokens_used: Option<u32>,
    pub finish_reason: Option<String>,
    pub raw_response_json: Option<String>,
}

// ---------------------------------------------------------------------------
// tool-call-event / tool-execution-result (wire names are kebab-case)
// ---------------------------------------------------------------------------

/// `tool-call-event` vocabulary record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEvent {
    #[serde(rename = "tool-name")]
    pub tool_name: String,
    #[serde(rename = "arguments-json")]
    pub arguments_json: String,
    #[serde(rename = "session-id")]
    pub session_id: Option<String>,
    #[serde(rename = "step-id")]
    pub step_id: u32,
}

/// `tool-execution-result` vocabulary record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    #[serde(rename = "tool-name")]
    pub tool_name: String,
    pub success: bool,
    #[serde(rename = "output-json")]
    pub output_json: String,
    #[serde(rename = "error-message")]
    pub error_message: Option<String>,
    #[serde(rename = "step-id")]
    pub step_id: u32,
}

/// `log-event` vocabulary record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub level: String,
    pub message: String,
    pub timestamp: Option<String>,
}

// ---------------------------------------------------------------------------
// Envelope / post-back
// ---------------------------------------------------------------------------

/// SSE event envelope (`type` + snake_case ids + opaque `payload`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(rename = "correlation_id")]
    pub correlation_id: Option<String>,
    #[serde(rename = "session_id")]
    pub session_id: Option<String>,
    #[serde(rename = "client_id")]
    pub client_id: String,
    pub payload: Value,
}

/// POST-back body for `POST /antikythera/v1/events/{correlation-id}/response`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostbackBody {
    #[serde(rename = "correlation_id")]
    pub correlation_id: String,
    pub ok: bool,
    pub payload: Value,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Event payload shapes (snake_case, per golden)
// ---------------------------------------------------------------------------

/// `hook-request` payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRequestPayload {
    pub hook: String,
    #[serde(rename = "session_state_json")]
    pub session_state_json: String,
    #[serde(rename = "input_json")]
    pub input_json: String,
}

/// `llm-token` payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmTokenPayload {
    #[serde(rename = "session_id")]
    pub session_id: Option<String>,
    pub chunk: String,
    #[serde(rename = "correlation_id")]
    pub correlation_id: Option<String>,
}

/// `lifecycle` payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecyclePayload {
    pub signal: String,
}

// ---------------------------------------------------------------------------
// ToolDefinition (registry shape, snake_case per golden)
// ---------------------------------------------------------------------------

/// Canonical tool definition used by `GET /tools` and `register-tools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
}

impl ToolDefinition {
    /// Build a minimal definition with an empty object input schema.
    pub fn simple(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            title: None,
            description: description.into(),
            parameters: Vec::new(),
            input_schema: Some(json!({"type": "object", "properties": {}, "required": []})),
            output_schema: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Mapping wire → runner contract (WIRE_PROTOCOL §6)
// ---------------------------------------------------------------------------

/// Maps a wire `tool-execution-result` into the runner `ToolResultInput`
/// JSON shape: `{tool_name, success, output_json, error_message?,
/// correlation_id?}`. `step_id` is dropped (the runner derives it from
/// session state); `output_json` is required and forwarded verbatim.
pub fn tool_execution_result_to_runner_input(
    result: &ToolExecutionResult,
    correlation_id: Option<String>,
) -> Value {
    json!({
        "tool_name": result.tool_name,
        "success": result.success,
        "output_json": result.output_json,
        "error_message": result.error_message,
        "correlation_id": correlation_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_execution_result_maps_to_runner_input() {
        let result = ToolExecutionResult {
            tool_name: "get_current_time".to_string(),
            success: true,
            output_json: "{\"datetime\":\"2026-08-12T00:00:00Z\"}".to_string(),
            error_message: None,
            step_id: 7,
        };
        let input = tool_execution_result_to_runner_input(&result, Some("corr-1".to_string()));
        assert_eq!(input["tool_name"], "get_current_time");
        assert_eq!(input["success"], true);
        assert_eq!(
            input["output_json"],
            "{\"datetime\":\"2026-08-12T00:00:00Z\"}"
        );
        assert!(input["error_message"].is_null());
        assert_eq!(input["correlation_id"], "corr-1");
        // step_id is dropped (runner derives it from session state)
        assert!(input.get("step_id").is_none());
        // output_json is required — the runner rejects the non-_json field
        assert!(input.get("output").is_none());
    }

    #[test]
    fn tool_execution_result_failure_keeps_error_and_correlation() {
        let result = ToolExecutionResult {
            tool_name: "rm".to_string(),
            success: false,
            output_json: "{}".to_string(),
            error_message: Some("permission denied".to_string()),
            step_id: 3,
        };
        let input = tool_execution_result_to_runner_input(&result, None);
        assert_eq!(input["tool_name"], "rm");
        assert_eq!(input["success"], false);
        assert_eq!(input["error_message"], "permission denied");
        assert!(input["correlation_id"].is_null());
    }

    #[test]
    fn tool_call_event_wire_names_are_kebab_case() {
        let event = ToolCallEvent {
            tool_name: "echo".to_string(),
            arguments_json: "{}".to_string(),
            session_id: Some("s1".to_string()),
            step_id: 1,
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["tool-name"], "echo");
        assert_eq!(json["arguments-json"], "{}");
        assert_eq!(json["session-id"], "s1");
        assert_eq!(json["step-id"], 1);
        // no snake_case leak
        assert!(json.get("tool_name").is_none());
    }

    #[test]
    fn tool_execution_result_wire_names_are_kebab_case() {
        let result = ToolExecutionResult {
            tool_name: "echo".to_string(),
            success: true,
            output_json: "{\"x\":1}".to_string(),
            error_message: None,
            step_id: 1,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["tool-name"], "echo");
        assert_eq!(json["output-json"], "{\"x\":1}");
        assert_eq!(json["step-id"], 1);
        assert!(json.get("output_json").is_none());
    }

    #[test]
    fn llm_request_and_response_wire_names_are_snake_case() {
        let request = LlmRequest {
            provider: Some("ollama".to_string()),
            model: Some("gpt-oss:120b-cloud".to_string()),
            session_id: Some("session-123".to_string()),
            messages_json: "[]".to_string(),
            force_json: false,
            temperature: Some(0.7),
            max_tokens: Some(512),
            schema_name: None,
            metadata_json: None,
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["session_id"], "session-123");
        assert_eq!(json["messages_json"], "[]");
        assert_eq!(json["force_json"], false);
        assert_eq!(json["max_tokens"], 512);
        assert!(json.get("session-id").is_none());

        let response = LlmResponse {
            content: "Hello".to_string(),
            model: Some("gpt-oss:120b-cloud".to_string()),
            session_id: Some("session-123".to_string()),
            message_json: None,
            tokens_used: Some(4),
            finish_reason: Some("stop".to_string()),
            raw_response_json: None,
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["tokens_used"], 4);
        assert_eq!(json["finish_reason"], "stop");
        assert!(json.get("tokens-used").is_none());
    }

    #[test]
    fn envelope_shape_matches_golden() {
        let envelope = EventEnvelope {
            event_type: "tool-execution-request".to_string(),
            correlation_id: Some("corr-0001".to_string()),
            session_id: Some("session-123".to_string()),
            client_id: "client-a".to_string(),
            payload: json!({"tool-name": "get_current_time"}),
        };
        let json = serde_json::to_value(&envelope).unwrap();
        assert_eq!(json["type"], "tool-execution-request");
        assert_eq!(json["correlation_id"], "corr-0001");
        assert_eq!(json["session_id"], "session-123");
        assert_eq!(json["client_id"], "client-a");
        assert_eq!(json["payload"]["tool-name"], "get_current_time");
    }
}
