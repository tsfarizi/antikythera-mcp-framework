//! Server-side harness proving the composed composite.
//!
//! Default world (unchanged): a 3-way composite
//!
//!   dist/antikythera-sdk.wasm = antikythera-sdk (runner export, tool-registry
//!                               + logic-hooks imports) + antikythera-toolrunner
//!                               (tool-registry export with builtin `echo`)
//!                               + antikythera-default-hooks (logic-hooks
//!                               export, no-op passthrough)
//!
//! A5 extension: the component path is accepted as an optional CLI argument
//! (default `dist/antikythera-sdk.wasm`), and `--expect-final` switches the
//! assertions to the logic-hooks probe variant:
//!
//!   base   (default-hooks)  → commit action=call_tool, drain has
//!                             tool_requested + tool_result echo success
//!   custom (example hooks)  → decide-action forces action=final
//!                             content=hook-forced-final, NO tool_result echo
//!
//! B4 extension: the final probe is GENERIC — `--expect-content=<string>`
//! (default `hook-forced-final`, so the A5 invocation is unchanged) asserts
//! the commit envelope carries `action=final` + `content=<string>` and no
//! tool_result. The same init/prepare/commit/drain flow proves the drop-in
//! `logic-core-example` artifact (echo-agent: commit always returns
//! `{"action":"final","content":"echo-agent-done",...}`); its template holes
//! (`drain-events`, ...) return `Err` not-implemented, which the final probe
//! tolerates as "no events". `--expect=notimpl` probes those holes directly:
//! it calls an unimplemented runner function and asserts the structured
//! `{"error":"not implemented","function":"<name>"}` Err.
//!
//! Flow mirrored from tests/sdk/wasm_agent/deterministic_harness_tests.rs
//! (native Rust path), executed here against the WASM composite via the
//! wasmtime component API:
//!
//!   init(config) -> prepare-user-turn(user message)
//!   -> commit-llm-response(prepared, llm_response with call_tool echo)
//!   -> drain-events
//!
//! Usage:
//!   component-harness [COMPONENT_PATH] [--expect default|final|notimpl]
//!                     [--expect-content=<string>]

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Expect {
    /// Default behavior: call_tool commit + builtin echo tool_result.
    Default,
    /// Generic final probe: commit action=final + content=expect-content,
    /// no tool_result (A5 logic-hooks override or B4 logic-core echo commit).
    Final,
    /// B4 logic-core probe: call template-hole runner functions and assert
    /// the structured not-implemented Err.
    NotImpl,
}

impl std::fmt::Display for Expect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expect::Default => f.write_str("default"),
            Expect::Final => f.write_str("final"),
            Expect::NotImpl => f.write_str("notimpl"),
        }
    }
}

fn main() -> Result<()> {
    let mut expect = Expect::Default;
    let mut expect_content = "hook-forced-final".to_string();
    let mut component_path: Option<String> = None;
    for arg in std::env::args().skip(1) {
        if let Some(mode) = arg.strip_prefix("--expect=") {
            expect = match mode {
                "default" => Expect::Default,
                "final" => Expect::Final,
                "notimpl" => Expect::NotImpl,
                other => {
                    anyhow::bail!("unknown --expect mode: {other} (expected default|final|notimpl)")
                }
            };
        } else if let Some(content) = arg.strip_prefix("--expect-content=") {
            expect_content = content.to_string();
        } else if arg == "--expect-final" {
            expect = Expect::Final;
        } else if arg.starts_with('-') {
            anyhow::bail!("unknown flag: {arg}");
        } else if component_path.is_none() {
            component_path = Some(arg);
        } else {
            anyhow::bail!("unexpected extra argument: {arg}");
        }
    }
    let component_path = component_path.unwrap_or_else(|| {
        format!(
            "{}/../../dist/antikythera-sdk.wasm",
            env!("CARGO_MANIFEST_DIR")
        )
    });

    println!("=== component-harness: wasmtime composite server probe ===");
    println!("[config] component = {component_path}");
    println!("[config] expect     = {expect}");
    if expect == Expect::Final {
        println!("[config] content    = {expect_content}");
    }

    let engine = wasmtime::Engine::default();
    let mut linker = wasmtime::component::Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)
        .context("register wasi imports into linker")?;

    let component_bytes =
        std::fs::read(&component_path).context("read composite component path")?;
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

    // B4 not-implemented probe: template-hole functions of a logic-core
    // drop-in need no init/prepare/commit flow, so probe them directly.
    if expect == Expect::NotImpl {
        println!("--- not-implemented probe (logic-core drop-in) ---");

        let prompt_result = runner
            .call_get_tools_prompt(&mut store)
            .context("wasmtime trap on get-tools-prompt")?;
        match prompt_result {
            Ok(prompt) => anyhow::bail!(
                "FAIL: get-tools-prompt returned Ok({prompt:?}); expected structured \
                 not-implemented error"
            ),
            Err(e) => assert_not_implemented(&e, "get-tools-prompt")?,
        }

        let sweep_result = runner
            .call_sweep_idle_sessions(&mut store, None)
            .context("wasmtime trap on sweep-idle-sessions")?;
        match sweep_result {
            Ok(count) => anyhow::bail!(
                "FAIL: sweep-idle-sessions returned Ok({count}); expected structured \
                 not-implemented error"
            ),
            Err(e) => assert_not_implemented(&e, "sweep-idle-sessions")?,
        }

        println!("PASS: logic core template holes return structured not-implemented errors.");
        return Ok(());
    }

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
    let prepared: Value =
        serde_json::from_str(&prepared_json).context("prepared turn is not valid JSON")?;
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
    let commit: Value =
        serde_json::from_str(&commit_json).context("commit result is not valid JSON")?;
    println!(
        "[commit] action={} content={} tool={} fsm={}",
        commit["action"], commit["content"], commit["tool_name"], commit["fsm_state"]
    );

    // 4. drain-events(session_id)
    let drain_result = runner.call_drain_events(&mut store, &session_id);
    let events_json = match drain_result {
        Ok(Ok(json)) => json,
        // Logic-core drop-in: `drain-events` is a template hole
        // (Err not-implemented), so no event queue exists — the generic
        // final probe treats that as "no events" and continues to assert
        // that no tool_result was emitted. The default probe keeps the
        // strict error propagation.
        Ok(Err(e)) if expect == Expect::Final => {
            println!("[drain] Err tolerated by final probe (logic-core drop-in): {e}");
            "[]".to_string()
        }
        Ok(Err(e)) => anyhow::bail!("drain-events failed: {e}"),
        Err(trap) => anyhow::bail!("wasmtime trap on drain-events: {trap}"),
    };
    let events: Value = serde_json::from_str(&events_json).context("events are not valid JSON")?;

    let mut tool_requested_found = false;
    let mut tool_result_found = false;
    if let Some(events) = events.as_array() {
        for event in events {
            let kind = event["kind"].as_str().unwrap_or("?");
            println!(
                "[event] seq={} kind={} payload={}",
                event["seq"], kind, event["payload"]
            );
            if kind == "tool_requested" && event["payload"]["tool"] == "echo" {
                tool_requested_found = true;
            }
            if kind == "tool_result"
                && event["payload"]["tool"] == "echo"
                && event["payload"]["success"] == true
            {
                tool_result_found = true;
            }
        }
    } else {
        anyhow::bail!("drain-events did not return a JSON array: {events}");
    }

    let pass = match expect {
        Expect::Default => {
            // Baseline: SDK default behavior must be intact — the composite
            // behaves exactly as the pre-hook composite did (default-hooks is
            // a no-op passthrough, so the third member changes no behavior).
            if commit["action"] != "call_tool" {
                anyhow::bail!("expected commit action=call_tool, got {}", commit["action"]);
            }
            if !tool_requested_found {
                anyhow::bail!("FAIL: drain-events has no tool_requested event for `echo`");
            }
            if !tool_result_found {
                anyhow::bail!(
                    "FAIL: drain-events has no successful tool_result event for builtin `echo` \
                     (no host round-trip expected)"
                );
            }
            println!(
                "PASS: default hooks — builtin tool `echo` flowed through \
                 tool_requested + tool_result(success=true) inside the composite."
            );
            true
        }
        Expect::Final => {
            // Generic final probe (B4): the commit envelope MUST carry
            // action=final + content=expect-content — either the A5
            // logic-hooks decide-action override (hook-forced-final) or the
            // B4 logic-core deterministic echo-agent commit
            // (echo-agent-done). The tool MUST NOT execute, so no
            // tool_result echo may appear.
            if commit["action"] != "final" {
                anyhow::bail!(
                    "FAIL: expected commit action=final (decide-action override), got {}",
                    commit["action"]
                );
            }
            if commit["content"] != expect_content {
                anyhow::bail!(
                    "FAIL: expected commit content={expect_content:?}, got {}",
                    commit["content"]
                );
            }
            if tool_result_found {
                anyhow::bail!(
                    "FAIL: final probe — tool_result echo present despite action=final; \
                     tool executed when the final decision should have prevented it"
                );
            }
            if tool_requested_found {
                println!(
                    "[note] tool_requested event observed: the SDK emits it during default \
                     action derivation before decide-action is consulted (commit path); no \
                     tool_result follows, so the tool did not execute."
                );
            }
            println!(
                "PASS: final probe — commit action=final content={expect_content:?}; \
                 no tool_result (tool never executed)."
            );
            true
        }
        // The notimpl probe returns before the init/prepare/commit flow.
        Expect::NotImpl => unreachable!("notimpl probe handled above"),
    };

    if !pass {
        std::process::exit(1);
    }
    Ok(())
}

/// B4 structured-error assertion: the Err string must be the JSON object
/// `{"error":"not implemented","function":"<name>"}` produced by a logic-core
/// template hole.
fn assert_not_implemented(err: &str, function: &str) -> Result<()> {
    let parsed: Value = serde_json::from_str(err)
        .with_context(|| format!("not-implemented error is not valid JSON: {err}"))?;
    if parsed["error"] != "not implemented" {
        anyhow::bail!(
            "FAIL: expected error.error=\"not implemented\" for {function}, got {}",
            parsed["error"]
        );
    }
    if parsed["function"] != function {
        anyhow::bail!(
            "FAIL: expected error.function=\"{function}\", got {}",
            parsed["function"]
        );
    }
    println!("[notimpl] {function}: Err {err}");
    Ok(())
}
