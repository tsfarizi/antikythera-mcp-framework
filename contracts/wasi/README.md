# WASI Component Contract

This directory contains the contract definition for the WASI Component Model target (`wasm32-wasip1`).

## Contract File

- `../shared/wit_signatures.golden.txt` — WIT function signatures (golden file)

## Build Target

- `wasm32-wasip1`
- Feature flag: `component`
- Build command: `cargo component build -p antikythera-sdk --release --target wasm32-wasip1 --no-default-features --features component`

## Interface

The WASI component uses WIT (WebAssembly Interface Types) defined in `wit/antikythera.wit`.

### Imports (host provides)
- `host-imports`: `call-llm`, `emit-tool-call`, `log-message`, `save-state`, `load-state`

### Exports (WASM provides)
- `prompt-manager`: `get-prompt`, `list-prompts`
- `mcp-client`: `list-tools`, `invoke-tool`
- `ffi-server`: `start`, `stop`

## Verification

```bash
cargo test -p antikythera-tests --test compatibility_tests -- wit_contract_signatures_match_golden
```
