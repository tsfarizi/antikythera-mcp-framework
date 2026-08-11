//! # logic-core-template
//!
//! Starter crate for a host-authored drop-in `runner` component
//! (`antikythera:agent-sdk/runner`, world `logic-core-component`).
//!
//! The exported `runner` interface is IDENTICAL to the composite SDK's
//! (world `antikythera-agent-sdk`): the same 16 functions, the same
//! JSON-string semantics (payload contract in
//! `contracts/shared/payload_contract.golden.json`), so a host that loads
//! the composite SDK loads this component with zero host code changes.
//!
//! ## How to author a logic core
//!
//! Edit ONLY the `custom_*` functions below. The WIT export adapter in
//! `component.rs` (compiled with the `component` feature) is not touched.
//!
//! Every `custom_*` hook defaults to `None`, which means "use the template
//! default for this runner function". The adapter maps each function as
//! follows:
//!
//! | runner function | custom hook | `None` → template default |
//! |---|---|---|
//! | init | [`custom_init`] | create/reuse the in-memory session, return the session id |
//! | prepare-user-turn | [`custom_prepare_turn`] | `Err` not-implemented |
//! | commit-llm-response | [`custom_commit_response`] | `Err` not-implemented |
//! | commit-llm-stream | [`custom_commit_stream`] | `Err` not-implemented |
//! | process-llm-response-for-session | [`custom_process_llm_response`] | `Err` not-implemented |
//! | process-tool-result-for-session | [`custom_process_tool_result`] | `Err` not-implemented |
//! | append-llm-chunk | [`custom_append_chunk`] | `Err` not-implemented |
//! | drain-events | [`custom_drain_events`] | `Err` not-implemented |
//! | get-state | — (fixed store plumbing) | in-memory state JSON, or `Session not found: <id>` |
//! | reset-session | — (fixed store plumbing) | drop the session, `Ok(existed)` |
//! | sweep-idle-sessions | [`custom_sweep_idle`] | `Err` not-implemented |
//! | register-tools | [`custom_register_tools`] | `Err` not-implemented |
//! | get-tools-prompt | [`custom_get_tools_prompt`] | `Err` not-implemented |
//! | set-context-policy | [`custom_set_context_policy`] | `Err` not-implemented |
//! | get-telemetry-snapshot | [`custom_get_telemetry`] | `Err` not-implemented |
//! | get-slo-snapshot | [`custom_get_slo`] | `Err` not-implemented |
//!
//! The structured not-implemented error is the single consistent format
//! `{"error":"not implemented","function":"<kebab-case-name>"}`, produced by
//! the adapter in `component.rs` whenever a `custom_*` hook is `None` and no
//! template default exists. Host code can detect "template hole"
//! deterministically by checking for the `"error"` field.
//!
//! The template ships with exactly the minimal in-memory state the task
//! requires (`init` / `get-state` / `reset-session`); every other runner
//! function is a hook the host author fills in.
//!
//! Build the component:
//!
//! ```text
//! cargo component build -p logic-core-template --release --target wasm32-wasip1 --no-default-features --features component
//! ```
//!
//! The artifact exports `antikythera:agent-sdk/runner@1.0.0` (16 functions).
//! The world declares optional `host-imports` and `tool-registry` imports;
//! a template that never references them ends up importing nothing (the
//! component encoder prunes unreferenced imports).

/// Custom `init` behavior.
///
/// TODO: isi logika host (mis. parse config, siapkan session, return session id).
/// `None` = template default: parse `config-json` leniently, create/reuse an
/// in-memory session, and return the session id — the same return contract as
/// the SDK runner (a bare id string, not a JSON object).
pub fn custom_init(_config_json: &str) -> Option<String> {
    None
}

/// Custom `prepare-user-turn` behavior.
///
/// TODO: isi logika host (mis. bangun messages, system prompt, tool block).
/// `None` = `Err` not-implemented (`{"error":"not implemented","function":"prepare-user-turn"}`).
pub fn custom_prepare_turn(_request_json: &str) -> Option<String> {
    None
}

/// Custom `commit-llm-response` behavior.
///
/// TODO: isi logika host (mis. proses respons, tentukan action, kembalikan
/// CommitResult envelope: action/content/tool_name/tool_input/session_id/step/fsm_state).
/// `None` = `Err` not-implemented.
pub fn custom_commit_response(
    _prepared_turn_json: &str,
    _llm_response_json: &str,
) -> Option<String> {
    None
}

/// Custom `commit-llm-stream` behavior.
///
/// TODO: isi logika host (mis. gabungkan chunk pending lalu commit).
/// `None` = `Err` not-implemented.
pub fn custom_commit_stream(_prepared_turn_json: &str) -> Option<String> {
    None
}

/// Custom `process-llm-response-for-session` behavior.
///
/// TODO: isi logika host (kembalikan serialized AgentAction:
/// `{"action":"call_tool",...}` | `{"action":"final","response":...}` | `{"action":"retry","error":...}`).
/// `None` = `Err` not-implemented.
pub fn custom_process_llm_response(_session_id: &str, _llm_response_json: &str) -> Option<String> {
    None
}

/// Custom `process-tool-result-for-session` behavior.
///
/// TODO: isi logika host (terima ToolResultInput:
/// tool_name/success/output_json/error_message/correlation_id; kembalikan
/// `{session_id, step, next_message, tool_result}`).
/// `None` = `Err` not-implemented.
pub fn custom_process_tool_result(_session_id: &str, _tool_result_json: &str) -> Option<String> {
    None
}

/// Custom `append-llm-chunk` behavior.
///
/// TODO: isi logika host (buffer chunk streaming per session).
/// `None` = `Err` not-implemented.
pub fn custom_append_chunk(
    _session_id: &str,
    _chunk: &str,
    _correlation_id: Option<&str>,
) -> Option<bool> {
    None
}

/// Custom `drain-events` behavior.
///
/// TODO: isi logika host (kembalikan JSON array StreamEvent:
/// `{seq, session_id, step, correlation_id, kind, payload}`).
/// `None` = `Err` not-implemented.
pub fn custom_drain_events(_session_id: &str) -> Option<String> {
    None
}

/// Custom `sweep-idle-sessions` behavior.
///
/// TODO: isi logika host (reap sesi idle, kembalikan jumlah).
/// `None` = `Err` not-implemented.
pub fn custom_sweep_idle(_now_unix_ms: Option<i64>) -> Option<u32> {
    None
}

/// Custom `register-tools` behavior.
///
/// TODO: isi logika host (validasi array ToolDefinition, simpan katalog,
/// kembalikan jumlah).
/// `None` = `Err` not-implemented.
pub fn custom_register_tools(_tools_json: &str) -> Option<u32> {
    None
}

/// Custom `get-tools-prompt` behavior.
///
/// TODO: isi logika host (kembalikan blok tool-list siap suntik ke system prompt).
/// `None` = `Err` not-implemented.
pub fn custom_get_tools_prompt() -> Option<String> {
    None
}

/// Custom `set-context-policy` behavior.
///
/// TODO: isi logika host (terima envelope `{"policy": {...}}`).
/// `None` = `Err` not-implemented.
pub fn custom_set_context_policy(_policy_json: &str) -> Option<bool> {
    None
}

/// Custom `get-telemetry-snapshot` behavior.
///
/// TODO: isi logika host (kembalikan TelemetrySnapshot JSON:
/// session_id/correlation_id/counters/total_prepare_latency_ms/total_commit_latency_ms/fsm_state).
/// `None` = `Err` not-implemented.
pub fn custom_get_telemetry(_session_id: &str) -> Option<String> {
    None
}

/// Custom `get-slo-snapshot` behavior.
///
/// TODO: isi logika host (kembalikan SloSnapshot JSON:
/// session_id/correlation_id/success_rate/tool_error_rate/retry_ratio/p95_*).
/// `None` = `Err` not-implemented.
pub fn custom_get_slo(_session_id: &str) -> Option<String> {
    None
}

/// WIT export layer for the component world (`logic-core-component`).
#[cfg(feature = "component")]
pub mod component;
