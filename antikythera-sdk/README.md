# antikythera-sdk

High-level SDK and WASM component surface for the Antikythera Agent SDK.

## Features

- FFI-ready API with C-string helpers and unified `ffi_handler!` macro
- WASM agent runner with session lifecycle, LLM response processing, tool validation
- Prompt management FFI bindings
- JSON-based configuration serialization
- SDK logging with per-module loggers and query API
- Session and log re-exports for host integration

## Feature Flags

| Flag | Purpose | Status |
|:-----|:--------|:-------|
| `component` | Server-side WASM Component Model support (wasm32-wasip1 WASI) | Active |
| `wasm` | Browser WASM support (wasm32-unknown-unknown), enables `antikythera-log/wasm` | Active |
| `toolrunner` | In-process tool execution via `antikythera-toolrunner` | Active |
| `wasm-sandbox` | Wasmtime-based sandbox execution | Active |
