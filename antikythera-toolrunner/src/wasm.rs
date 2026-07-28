//! WASM bridge for tool execution.
//!
//! Provides a synchronous handler for the `emit-tool-call` host-import
//! that intercepts tool calls and executes builtin tools in-process,
//! eliminating the round-trip to the host for registered tools.
//!
//! ## Flow
//!
//! ```text
//! WASM Agent: emit-tool-call(event)
//!   │
//!   ▼
//! handle_tool_call(&runner, event)
//!   │
//!   ├── runner.is_builtin(tool_name)?
//!   │     YES → runner.execute(tool_name, args)  ← in-process, zero overhead
//!   │     NO  → return Err(HostRequired)         ← host must handle
//!   │
//!   └── return ToolResult
//! ```

use crate::runner::ToolRunner;
use crate::types::{ToolCall, ToolResult};

/// Handle an `emit-tool-call` host-import event.
///
/// If the tool has a builtin handler in the runner, it is executed in-process.
/// Returns `Err(HostRequired)` if the tool is not builtin — the caller
/// should delegate to the host.
///
/// # Arguments
///
/// * `runner` — The `ToolRunner` with registered tools and handlers.
/// * `tool_name` — Name of the tool to call.
/// * `arguments` — JSON arguments for the tool.
/// * `step_id` — Current agent step number.
pub fn handle_tool_call(
    runner: &ToolRunner,
    tool_name: &str,
    arguments: serde_json::Value,
    step_id: u32,
) -> Result<ToolResult, crate::error::ToolRunnerError> {
    let call = ToolCall {
        name: tool_name.to_string(),
        arguments,
        step_id,
    };
    runner.execute_call(&call)
}

/// Build a JSON payload compatible with the SDK's `process_tool_result_for_session`.
///
/// This bridges the `ToolRunner`'s `ToolResult` to the format expected by
/// the WASM agent runner's tool pipeline.
pub fn tool_result_to_json(result: &ToolResult) -> String {
    serde_json::to_string(result).unwrap_or_else(|_| {
        serde_json::json!({
            "name": result.name,
            "success": result.success,
            "output": result.output,
            "error": result.error,
        })
        .to_string()
    })
}

/// Create a `ToolRunner` from a JSON array of tool definitions with no handlers.
///
/// Useful for validation-only scenarios (e.g., validating LLM tool calls
/// before delegating execution to the host).
pub fn registry_only_runner(
    definitions_json: &str,
) -> Result<ToolRunner, crate::error::ToolRunnerError> {
    let mut runner = ToolRunner::new();
    runner.load_definitions(definitions_json)?;
    Ok(runner)
}
