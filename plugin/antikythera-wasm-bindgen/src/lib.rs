use wasm_bindgen::prelude::*;

/// Convert runner errors into JS exceptions carrying the error message.
fn err_to_js(e: antikythera_sdk::wasm_agent::runner::AgentRunnerError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

// ── Session lifecycle ─────────────────────────────────────────────

#[wasm_bindgen]
pub fn init(config_json: &str) -> Result<String, JsValue> {
    antikythera_sdk::wasm_agent::runner::init(config_json).map_err(err_to_js)
}

#[wasm_bindgen]
pub fn prepare_user_turn(request_json: &str) -> Result<String, JsValue> {
    antikythera_sdk::wasm_agent::runner::prepare_user_turn(request_json).map_err(err_to_js)
}

#[wasm_bindgen]
pub fn commit_llm_response(
    prepared_turn_json: &str,
    llm_response_json: &str,
) -> Result<String, JsValue> {
    antikythera_sdk::wasm_agent::runner::commit_llm_response(prepared_turn_json, llm_response_json)
        .map_err(err_to_js)
}

#[wasm_bindgen]
pub fn commit_llm_stream(prepared_turn_json: &str) -> Result<String, JsValue> {
    antikythera_sdk::wasm_agent::runner::commit_llm_stream(prepared_turn_json).map_err(err_to_js)
}

#[wasm_bindgen]
pub fn process_llm_response_for_session(
    session_id: &str,
    llm_response_json: &str,
) -> Result<String, JsValue> {
    antikythera_sdk::wasm_agent::runner::process_llm_response_for_session(
        session_id,
        llm_response_json,
    )
    .map_err(err_to_js)
}

#[wasm_bindgen]
pub fn process_tool_result_for_session(
    session_id: &str,
    tool_result_json: &str,
) -> Result<String, JsValue> {
    antikythera_sdk::wasm_agent::runner::process_tool_result_for_session(
        session_id,
        tool_result_json,
    )
    .map_err(err_to_js)
}

#[wasm_bindgen]
pub fn append_llm_chunk(
    session_id: &str,
    chunk: &str,
    correlation_id: Option<String>,
) -> Result<bool, JsValue> {
    antikythera_sdk::wasm_agent::runner::append_llm_chunk(
        session_id,
        chunk,
        correlation_id.as_deref(),
    )
    .map_err(err_to_js)
}

#[wasm_bindgen]
pub fn drain_events(session_id: &str) -> Result<String, JsValue> {
    antikythera_sdk::wasm_agent::runner::drain_events(session_id).map_err(err_to_js)
}

#[wasm_bindgen]
pub fn get_state(session_id: &str) -> Result<String, JsValue> {
    antikythera_sdk::wasm_agent::runner::get_state(session_id).map_err(err_to_js)
}

#[wasm_bindgen]
pub fn reset_session(session_id: &str) -> Result<bool, JsValue> {
    antikythera_sdk::wasm_agent::runner::reset_session(session_id).map_err(err_to_js)
}

#[wasm_bindgen]
pub fn sweep_idle_sessions(now_unix_ms: Option<i64>) -> Result<u32, JsValue> {
    antikythera_sdk::wasm_agent::runner::sweep_idle_sessions(now_unix_ms).map_err(err_to_js)
}

// ── Tool registry ─────────────────────────────────────────────────

#[wasm_bindgen]
pub fn register_tools(tools_json: &str) -> Result<u32, JsValue> {
    antikythera_sdk::wasm_agent::runner::register_tools(tools_json).map_err(err_to_js)
}

#[wasm_bindgen]
pub fn get_tools_prompt() -> Result<String, JsValue> {
    antikythera_sdk::wasm_agent::runner::get_tools_prompt().map_err(err_to_js)
}

// ── Configuration ─────────────────────────────────────────────────

#[wasm_bindgen]
pub fn set_context_policy(policy_json: &str) -> Result<bool, JsValue> {
    antikythera_sdk::wasm_agent::runner::set_context_policy(policy_json).map_err(err_to_js)
}

// ── Telemetry / observability ─────────────────────────────────────

#[wasm_bindgen]
pub fn get_telemetry_snapshot(session_id: &str) -> Result<String, JsValue> {
    antikythera_sdk::wasm_agent::runner::get_telemetry_snapshot(session_id).map_err(err_to_js)
}

#[wasm_bindgen]
pub fn get_slo_snapshot(session_id: &str) -> Result<String, JsValue> {
    antikythera_sdk::wasm_agent::runner::get_slo_snapshot(session_id).map_err(err_to_js)
}
