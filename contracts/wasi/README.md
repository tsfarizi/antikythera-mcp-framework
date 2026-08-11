# WASI Component Contract

This directory contains the contract definition for the WASI Component Model target (`wasm32-wasip1`).

## Contract Files

- `wit_signatures.golden.txt` — WIT function signatures (golden file, mechanically regenerated from `wit/antikythera.wit` in the same deterministic style as `contracts/shared/wit_signatures.golden.txt`)
- `component_contract.golden.json` — legacy JSON contract artifact (superseded; kept for history)

## Build Target

- `wasm32-wasip1`
- Feature flag: `component`
- Build command: `cargo component build -p antikythera-sdk --release --target wasm32-wasip1 --no-default-features --features component`

## Interface

The WASI component uses WIT (WebAssembly Interface Types) defined in `wit/antikythera.wit`. The deliverable is **composite** (three members): the SDK component (`world antikythera-agent-sdk`, exports `runner`) is composed with the standalone toolrunner component (`world tool-registry-component`, exports `tool-registry`) and the default-hooks component (`world logic-hooks-component`, exports `logic-hooks`) via `wasm-tools compose` into `dist/antikythera-sdk.wasm`.

### World `antikythera-agent-sdk` (SDK component)

- **Import** `tool-registry`: `list-tools-json`, `validate-tool-call`, `execute-builtin` — supplied by the embedded toolrunner component at composition time (stateless tool catalog + builtin executor; JSON-string payloads)
- **Import** `logic-hooks`: `prepare-turn`, `decide-action`, `handle-tool-result` — supplied by the embedded hooks component at composition time (no-op passthrough in the default deliverable; a host-authored component can be composed in its place)
- **Export** `runner`: `init`, `prepare-user-turn`, `commit-llm-response`, `commit-llm-stream`, `process-llm-response-for-session`, `process-tool-result-for-session`, `append-llm-chunk`, `drain-events`, `get-state`, `reset-session`, `sweep-idle-sessions`, `register-tools`, `get-tools-prompt`, `set-context-policy`, `get-telemetry-snapshot`, `get-slo-snapshot`

### World `tool-registry-component` (toolrunner component)

- **Export** `tool-registry` — the interface imported by the SDK world; `wasm-tools compose` wires this export to the SDK import

### World `logic-hooks-component` (default-hooks component)

- **Export** `logic-hooks` — the interface imported by the SDK world; the default deliverable embeds `antikythera-default-hooks` (no-op passthrough provider), `wasm-tools compose` wires this export to the SDK import

### Import resolution

The standalone SDK component carries unmet `tool-registry` and `logic-hooks` imports and must never be embedded or transpiled directly. The composed composite imports only `wasi:` interfaces and exports `antikythera:agent-sdk/runner@1.0.0`.

## Verification

```bash
wasm-tools component wit dist/antikythera-sdk.wasm
cargo run -p component-harness --release
```
