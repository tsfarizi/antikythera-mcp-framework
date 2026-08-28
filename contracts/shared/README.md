# Shared Contract Schemas

This directory contains contract schemas shared across the WASI and browser targets. Both targets now ship the same WASI component (`wasm32-wasip2`); the browser target consumes it via jco-transpiled JS bindings.

## Contract Files

- `payload_contract.golden.json` — JSON payload key shapes (golden file)
- `wit_signatures.golden.txt` — WIT function signatures (reference for WASI target)

## Payload Contracts

The following JSON structures are shared between WASI and Browser targets:

### Prepared Turn

Keys: `correlation_id`, `force_json`, `messages_json`, `metadata_json`, `prompt`, `session_id`, `step`, `summary_handoff`, `system_prompt`

### Commit Result

Keys: `action`, `content`, `fsm_state`, `session_id`, `step`, `tool_input`, `tool_name`

### Tool Result

Keys: `next_message`, `session_id`, `step`, `tool_result`

### Tool Result Inner

Keys: `error`, `name`, `output`, `step_id`, `success`

## Verification

The golden artifacts are mechanically guarded by `tests/compatibility_tests.rs`:

```bash
cargo test -p antikythera-tests --test compatibility_tests -- browser_type_signatures_match_golden
cargo test -p antikythera-tests --test compatibility_tests -- payload_contract_shapes_match_golden
```

`payload_contract_shapes_match_golden` checks artifact integrity (valid JSON, exact top-level entries, non-empty `type`/`fields`). Payload *semantics* (key names of the wire JSON produced by `wasm_agent`) are verified by struct inspection of `antikythera-sdk/src/wasm_agent`; when the wasm agent's payload shapes change, update this golden in the same change.
