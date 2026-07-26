# Browser WASM Contract

This directory contains the contract definition for the browser WASM target (`wasm32-unknown-unknown`).

## Contract Files

- `antikythera_wasm_bindgen.d.ts` — TypeScript type definitions (golden file)
- `browser_function_signatures.golden.txt` — Expected exported function signatures

## Build Target

- `wasm32-unknown-unknown`
- Feature flag: `wasm`
- Build command: `cargo build -p antikythera-sdk --release --target wasm32-unknown-unknown --no-default-features --features wasm`

## Interface

The browser WASM uses `wasm-bindgen` to export functions to JavaScript. All functions accept and return JSON strings.

### Exported Functions

| Function | Parameters | Return |
|----------|-----------|--------|
| `init` | `config_json: string` | `string` |
| `prepare_user_turn` | `request_json: string` | `string` |
| `commit_llm_response` | `prepared_turn_json: string, llm_response_json: string` | `string` |
| `commit_llm_stream` | `prepared_turn_json: string` | `string` |
| `process_llm_response_for_session` | `session_id: string, llm_response_json: string` | `string` |
| `process_tool_result_for_session` | `session_id: string, tool_result_json: string` | `string` |
| `append_llm_chunk` | `session_id: string, chunk: string, correlation_id?: string` | `boolean` |
| `drain_events` | `session_id: string` | `string` |
| `get_state` | `session_id: string` | `string` |
| `reset_session` | `session_id: string` | `boolean` |
| `sweep_idle_sessions` | `now_unix_ms?: bigint` | `number` |
| `register_tools` | `tools_json: string` | `number` |
| `get_tools_prompt` | — | `string` |
| `set_context_policy` | `policy_json: string` | `boolean` |
| `get_telemetry_snapshot` | `session_id: string` | `string` |
| `get_slo_snapshot` | `session_id: string` | `string` |

## Verification

```bash
cargo test -p antikythera-tests --test compatibility_tests -- browser_type_signatures_match_golden
```
