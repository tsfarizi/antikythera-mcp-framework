//! # logic-hooks-example
//!
//! Deterministic probe crate: a filled-in copy of `logic-hooks-template`
//! proving that a host-authored override changes SDK behavior. Only
//! `custom_decide_action` differs from the template — the other two hooks
//! stay passthrough.
//!
//! ## Probe semantics
//!
//! `decide-action` ignores both inputs and always returns
//! `{"action":"final","content":"hook-forced-final"}`. The SDK merges this
//! object over its default commit-result envelope, so whichever LLM response
//! is committed, the resulting action is `final` — never `call_tool` or
//! `continue`. No real LLM is involved: the behavior is a pure function of
//! the component's exported `decide-action`.
//!
//! Build the component:
//!
//! ```text
//! cargo component build -p logic-hooks-example --release --target wasm32-wasip2 --no-default-features --features component
//! ```
//!
//! The artifact exports `antikythera:agent-sdk/logic-hooks@1.0.0` and can be
//! composed into the SDK with `wasm-tools compose`.

/// Custom `prepare-turn` behavior (unchanged from template: passthrough).
pub fn custom_prepare_turn(_request_json: &str, _session_state_json: &str) -> Option<String> {
    None
}

/// Custom `decide-action` behavior (the probe).
///
/// Returns a constant override regardless of the committed LLM response:
/// `action` is authoritative in the merged commit-result envelope, so the
/// session always commits a terminal `final` action with the forced content.
pub fn custom_decide_action(_session_state_json: &str, _llm_response_json: &str) -> Option<String> {
    Some(r#"{"action":"final","content":"hook-forced-final"}"#.to_string())
}

/// Custom `handle-tool-result` behavior (unchanged from template: passthrough).
pub fn custom_handle_tool_result(
    _session_state_json: &str,
    _tool_result_json: &str,
) -> Option<String> {
    None
}

/// WIT export layer for the component world (`logic-hooks-component`).
#[cfg(feature = "component")]
pub mod component;
