# Deprecation Policy

This policy defines current deprecation handling for public APIs.

## Lifecycle Flow

```mermaid
flowchart LR
    Introduce[Introduce replacement] --> Mark[Mark deprecated API]
    Mark --> Warn[Emit compile-time warning]
    Warn --> Migrate[Consumer migration]
    Migrate --> Remove[Major-version removal]
```

## Policy Rules

- A replacement API must exist before deprecation is introduced.
- Deprecated APIs must include `since` metadata and migration notes.
- Deprecated APIs remain thin delegates only.
- Removal occurs only on a major version boundary.

## Enforcement

- CI/lint gate for production targets:
  - `cargo clippy --workspace --lib --bins -- -D warnings -D deprecated`
- Backward-compatibility tests can explicitly allow deprecated paths when required.

## Current Deprecations

| API / Path | Since | Replaced by | Compatibility commitment |
|:-----------|:------|:------------|:-------------------------|
| Browser wasm-bindgen path (`wasm32-unknown-unknown` + `wasm` feature, `plugin/antikythera-wasm-bindgen`, wasm-pack) | WASI-component transition (feature `component` + `@bytecodealliance/jco`) | WASI component (`wasm32-wasip1`) transpiled with jco → `npm/antikythera-sdk/component/` (ESM, namespace `runner`) | The `./antikythera_wasm_bindgen` npm export and the `wasm` feature remain available during the transition; removal only on a major version boundary |

Migration notes:

- New browser integrations should `import { runner } from 'antikythera-agent/component'` instead of `import init from 'antikythera-agent/antikythera_wasm_bindgen'`.
- The Rust `wasm` feature (`antikythera-log/wasm`, js-sys time) is retained for crate-level compatibility but is no longer the browser path.
- Contract verification for the replacement path: `cargo test -p antikythera-tests --test compatibility_tests`.
