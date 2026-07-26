# Shared Contract Schemas

This directory contains contract schemas that apply to both WASI and Browser WASM targets.

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

```bash
cargo test -p antikythera-tests --test compatibility_tests -- payload_contract_shapes_match_golden
```
