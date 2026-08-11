//! Server-side harness proving the composed composite:
//!
//!   dist/antikythera-sdk.wasm = antikythera-sdk (runner export, tool-registry
//!                               import) + antikythera-toolrunner (tool-registry
//!                               export with builtin `echo`)
//!
//! Flow mirrored from tests/sdk/wasm_agent/deterministic_harness_tests.rs
//! (native Rust path), executed here against the WASM composite via the
//! wasmtime component API:
//!
//!   init(config) -> prepare-user-turn(user message)
//!   -> commit-llm-response(prepared, llm_response with call_tool echo)
//!   -> drain-events
//!
//! Assertion: drain-events MUST contain a ToolResult event with success=true
//! produced WITHOUT host round-trip, because `echo` is executed inside the
//! composite by the embedded toolrunner component.

use anyhow::{Context, Result};
use serde_json::Value;

wasmtime::component::bindgen!({
    path: "wit/harness.wit",
    world: "harness",
    require_store_data_send: true,
});

/// Host state: WASI context + resource table, per wasmtime-wasi 36 pattern.
struct WasiState {
    ctx: wasmtime_wasi::WasiCtx,
    table: wasmtime_wasi::ResourceTable,
}

impl wasmtime_wasi::WasiView for WasiState {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        wasmtime_wasi::WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

fn main() -> Result<()> {
    println!("=== component-harness: wasmtime composite server smoke ===");

    let engine = wasmtime::Engine::default();
    let mut linker = wasmtime::component::Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)
        .context("register wasi imports into linker")?;

    let component_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../dist/antikythera-sdk.wasm");
    let component_bytes =
        std::fs::read(component_path).context("read dist/antikythera-sdk.wasm")?;
    let component = wasmtime::component::Component::new(&engine, &component_bytes)
        .context("compile composite component")?;

    let mut store = wasmtime::Store::new(
        &engine,
        WasiState {
            ctx: wasmtime_wasi::WasiCtxBuilder::new().build(),
            table: wasmtime_wasi::ResourceTable::new(),
        },
    );

    let root = Harness::instantiate(&mut store, &component, &linker)
        .context("instantiate composite runner export")?;
    let runner = root.antikythera_agent_sdk_runner();

    // 1. init(config)
    let config_json = serde_json::json!({
        "session_id": "compose-smoke",
        "max_steps": 5,
        "auto_execute_tools": true,
    })
    .to_string();
    let session_id = runner
        .call_init(&mut store, &config_json)
        .context("wasmtime trap on init")?
        .map_err(|e| anyhow::anyhow!("runner init failed: {e}"))?;
    println!("[init] session_id = {session_id}");

    // 2. prepare-user-turn(user message)
    let request_json = serde_json::json!({
        "prompt": "echo hello",
        "session_id": session_id,
        "correlation_id": "compose-smoke-1",
    })
    .to_string();
    let prepared_json = runner
        .call_prepare_user_turn(&mut store, &request_json)
        .context("wasmtime trap on prepare-user-turn")?
        .map_err(|e| anyhow::anyhow!("prepare-user-turn failed: {e}"))?;
    let prepared: Value = serde_json::from_str(&prepared_json)
        .context("prepared turn is not valid JSON")?;
    println!(
        "[prepare] step={} prompt={}",
        prepared["step"], prepared["prompt"]
    );

    // 3. commit-llm-response(prepared, llm_response with call_tool -> echo)
    let llm_response_json = serde_json::json!({
        "action": "call_tool",
        "tool": "echo",
        "input": {"hello": "world"},
    })
    .to_string();
    let commit_json = runner
        .call_commit_llm_response(&mut store, &prepared_json, &llm_response_json)
        .context("wasmtime trap on commit-llm-response")?
        .map_err(|e| anyhow::anyhow!("commit-llm-response failed: {e}"))?;
    let commit: Value = serde_json::from_str(&commit_json).context("commit result is not valid JSON")?;
    println!(
        "[commit] action={} tool={} fsm={}",
        commit["action"],
        commit["tool_name"],
        commit["fsm_state"]
    );
    if commit["action"] != "call_tool" {
        anyhow::bail!("expected commit action=call_tool, got {}", commit["action"]);
    }

    // 4. drain-events(session_id) — assert builtin tool result surfaced.
    let events_json = runner
        .call_drain_events(&mut store, &session_id)
        .context("wasmtime trap on drain-events")?
        .map_err(|e| anyhow::anyhow!("drain-events failed: {e}"))?;
    let events: Value = serde_json::from_str(&events_json).context("events are not valid JSON")?;

    let mut tool_result_found = false;
    if let Some(events) = events.as_array() {
        for event in events {
            println!(
                "[event] seq={} kind={} payload={}",
                event["seq"], event["kind"], event["payload"]
            );
            if event["kind"] == "tool_result"
                && event["payload"]["tool"] == "echo"
                && event["payload"]["success"] == true
            {
                tool_result_found = true;
            }
        }
    } else {
        anyhow::bail!("drain-events did not return a JSON array: {events}");
    }

    if !tool_result_found {
        anyhow::bail!(
            "FAIL: drain-events has no successful tool_result event for builtin `echo` \
             (no host round-trip expected)"
        );
    }

    println!(
        "PASS: builtin tool `echo` executed inside the composite \
         (toolrunner component) with success=true, no host round-trip."
    );
    Ok(())
}
