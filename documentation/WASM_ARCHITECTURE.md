# WASM Architecture

This document describes the three WASM integration paths in the Antikythera MCP Framework, their target platforms, feature flags, contracts, and common pitfalls.

## Three WASM Paths

| Path | Target | Feature Flag | Build Command | Contract | When to Use |
|------|--------|-------------|---------------|----------|-------------|
| **WASI Component** | `wasm32-wasip1` | `component` | `cargo component build` | `wit/antikythera.wit` | Server-side: host embeds wasmtime and calls exports via FFI |
| **Browser** | `wasm32-unknown-unknown` | `wasm` | `cargo build` + wasm-bindgen | TypeScript `.d.ts` | Browser: JS host loads `.wasm` and calls exported functions |
| **Sandbox** | native | `wasm-sandbox` | `cargo build` | JSON over WASM memory | Host-side runner that loads pre-compiled WASM modules via wasmtime |

## Feature Flag Matrix

| Flag | Crate | Enables | Disables | Target |
|------|-------|---------|----------|--------|
| `component` | `antikythera-sdk` | `wasm_agent` module | — | `wasm32-wasip1` |
| `wasm` | `antikythera-sdk` | `antikythera-log/wasm` (js-sys time) | tokio runtime, filesystem | `wasm32-unknown-unknown` |
| `wasm-sandbox` | `antikythera-sdk` | wasmtime host runner | — | native |
| `subscriber` | `antikythera-log` | tokio + crossbeam-channel | — | native only |

### Dependency Chain

```
antikythera-wasm-bindgen
  → antikythera-sdk (features: component, no default-features)
    → antikythera-log (no wasm feature)

antikythera-sdk (features: wasm)
  → antikythera-log (features: wasm)
    → js-sys 0.3 (browser WASM time via Date.now())
    → wasm-bindgen 0.2 (JsValue type)
```

## Contracts per Target

### WASI Component (`wasm32-wasip1`)

**WIT interface** (`wit/antikythera.wit`):

- **Imports** (host provides):
  - `host-imports`: `call-llm`, `emit-tool-call`, `log-message`, `save-state`, `load-state`
- **Exports** (WASM provides):
  - `prompt-manager`: `get-prompt`, `list-prompts`
  - `mcp-client`: `list-tools`, `invoke-tool`
  - `ffi-server`: `start`, `stop`

**Build**: `cargo component build -p antikythera-sdk --release --target wasm32-wasip1 --no-default-features --features component`

### Browser (`wasm32-unknown-unknown`)

**wasm-bindgen exports** (17 functions, all accept/return JSON strings):

| Function | Purpose |
|----------|---------|
| `init` | Initialize agent runner with config |
| `prepare_user_turn` | Prepare LLM request from user input |
| `commit_llm_response` | Process completed LLM response |
| `commit_llm_stream` | Process streamed LLM response |
| `process_llm_response_for_session` | Process LLM response by session ID |
| `process_tool_result_for_session` | Process tool result by session ID |
| `append_llm_chunk` | Append streaming chunk to session |
| `drain_events` | Drain pending stream events |
| `get_state` | Get agent FSM state |
| `reset_session` | Reset a session |
| `sweep_idle_sessions` | Sweep idle sessions by timeout |
| `register_tools` | Register MCP tool definitions |
| `get_tools_prompt` | Get formatted tool list for prompts |
| `set_context_policy` | Set context management policy |
| `get_telemetry_snapshot` | Get telemetry counters |
| `get_slo_snapshot` | Get SLO metrics |

**Build**: `cargo build -p antikythera-sdk --release --target wasm32-unknown-unknown --no-default-features --features wasm`

### Sandbox (native)

**Exports expected from WASM module**: `antikythera_alloc`, `antikythera_run`, `memory`, `antikythera_dealloc`

**Host imports provided**: `call_llm_sync`

**Communication**: JSON over WASM memory (pointer+length packed as i64).

## WASM Time Abstraction (`wasm_compat`)

The `antikythera-log` crate provides `wasm_compat` module for platform-safe time:

| Target | Implementation | Mechanism |
|--------|---------------|-----------|
| Native (`not(wasm32)`) | `native.rs` | `chrono::Utc::now()` |
| Browser (`wasm32-unknown-unknown` + `wasm` feature) | `browser.rs` | `js_sys::Date::now()` |
| WASI (`wasm32-wasip1`) | `native.rs` | `chrono::Utc::now()` (WASI has system clock) |
| Browser without `wasm` feature | `native.rs` | `chrono::Utc::now()` (will panic at runtime) |

**Functions**:
- `now_unix_ms() -> i64` — Unix timestamp in milliseconds
- `now_rfc3339() -> String` — RFC 3339 formatted timestamp
- `now_timestamp_nanos() -> i64` — Unix timestamp in nanoseconds

**Why this exists**: `chrono::Utc::now()` calls `SystemTime::now()` which panics in `wasm32-unknown-unknown`. The `wasm` feature gates the browser-safe implementation using JavaScript's `Date.now()`.

## Common Pitfalls

### 1. `chrono::Utc::now()` panics in browser WASM

**Symptom**: Runtime panic when calling `chrono::Utc::now()` in `wasm32-unknown-unknown`.

**Fix**: Ensure `wasm` feature is enabled when building for browser. Use `antikythera_log::wasm_compat::now_unix_ms()` instead of `chrono::Utc::now().timestamp_millis()`.

### 2. Wrong feature flag for target

**Symptom**: Compilation error or runtime panic.

**Checklist**:
- `wasm32-wasip1` → use `component` feature
- `wasm32-unknown-unknown` → use `wasm` feature
- native → use `sdk-core` or `full` features

### 3. FSM parity drift

**Symptom**: WASM-side `AgentFsmState` diverges from core's `AgentFsmState`.

**Mitigation**: Run `fsm_parity_tests` — golden-file tests verify cross-crate FSM state parity.

### 4. Contract drift

**Symptom**: WIT signatures or JSON payload shapes change without updating consumers.

**Mitigation**: Run `wit_contract_signatures` and `payload_contract_shapes` tests — golden-file tests detect breaking changes.

## Build Optimization

```toml
[profile.wasm-release]
inherits = "release"
opt-level = "z"    # Size optimization
lto = true         # Link-time optimization
```

## CI Pipeline

The CI pipeline includes:
1. **Native test + clippy** — `cargo test --workspace` on ubuntu/windows/macos
2. **WASM compile check** — `cargo check` for both `wasm32-unknown-unknown` and `wasm32-wasip1`
3. **Contract tests** — WIT signatures, payload shapes, FSM parity
4. **Documentation build** — mdBook build
