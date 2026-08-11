# WASM Architecture

This document describes the WASM integration paths in the Antikythera Agent SDK, their target platforms, feature flags, contracts, and common pitfalls.

Currently, there are **two active WASM paths** (WASI Component for server, and the same component transpiled by `jco` for the browser), plus a **native sandbox** runner. The former `wasm-bindgen` browser path (`wasm32-unknown-unknown` + `wasm` feature) is **deprecated** and retained only for crate-level compatibility.

The WASI component deliverable is **composite**: `dist/antikythera-sdk.wasm` is produced by `wasm-tools compose`, wiring the `antikythera-sdk` component (exports `runner`, imports `tool-registry`) to the `antikythera-toolrunner` component (exports `tool-registry`, builtin tool execution). The standalone SDK component is an **intermediate artifact only** — it carries an unmet `tool-registry` import and must never be transpiled or embedded directly.

## WASM Paths

| Path | Target | Feature Flag | Build Command | Contract | When to Use |
|------|--------|-------------|---------------|----------|-------------|
| **WASI Component (server)** | `wasm32-wasip1` | `component` (both crates) | `task build` = `cargo component build -p antikythera-sdk` + `-p antikythera-toolrunner` + `wasm-tools compose` | `wit/antikythera.wit` (worlds `antikythera-agent-sdk` + `tool-registry-component`); composite exports `runner`, imports only WASI | Server-side: host embeds wasmtime and calls exports via FFI |
| **Browser (via jco)** | `wasm32-wasip1` composite → JS | `component` (Rust) + `@bytecodealliance/jco` (tooling) | `task build` (composite) + `task transpile` (`jco transpile` of the **composite**) | `npm/antikythera-sdk/component/` (ESM, namespace `runner`, camelCase) | Browser: JS host imports the transpiled ESM module and calls `runner` functions |
| **Sandbox** | native | `wasm-sandbox` | `cargo build` | JSON over WASM memory | Host-side runner that loads pre-compiled WASM modules via wasmtime |
| **wasm-bindgen (legacy)** | `wasm32-unknown-unknown` | `wasm` | `cargo build` + wasm-pack | TypeScript `.d.ts` | **Deprecated** — kept for `antikythera-log`/`plugin/antikythera-wasm-bindgen` compatibility during the transition |

## Feature Flag Matrix

| Flag | Crate | Enables | Disables | Target |
|------|-------|---------|----------|--------|
| `component` | `antikythera-sdk` | `wasm_agent` module + `wasm_exports` WIT export layer (`dep:wit-bindgen`) | — | `wasm32-wasip1` |
| `wasm` | `antikythera-sdk` | `antikythera-log/wasm` (js-sys time) — **legacy** for crate-level compatibility | tokio runtime, filesystem | `wasm32-unknown-unknown` (legacy) |
| `wasm` | `antikythera-log` | `wasm-bindgen` + `js-sys` (browser-safe time) — **legacy**, no longer the browser path | — | `wasm32-unknown-unknown` (legacy) |
| `wasm-sandbox` | `antikythera-sdk` | wasmtime host runner | — | native |
| `subscriber` | `antikythera-log` | tokio + crossbeam-channel | — | native only |
| `lint` | `antikythera-log` | compile-time lint blocking println!, eprintln!, dbg!, tracing | — | any |

> **jco** is not a Rust feature flag — it is npm tooling (`@bytecodealliance/jco`, installed at repo root) used by the browser path to transpile the component to JS.

### Dependency Chain

```
antikythera-sdk (features: component)
  → wit-bindgen (WIT bindings generator for the component world)
  → antikythera-log (no wasm feature)
  → imports antikythera:agent-sdk/tool-registry (world antikythera-agent-sdk)

antikythera-toolrunner (features: component)
  → wit-bindgen (WIT bindings generator for the component world)
  → exports antikythera:agent-sdk/tool-registry (world tool-registry-component,
    builtin tool execution — stateless, no host round-trip)

Composite deliverable, build-time tooling:
  wasm-tools compose target/.../antikythera_sdk.wasm \
    -d target/.../antikythera-toolrunner.wasm \
    -o dist/antikythera-sdk.wasm
  → wires the SDK's tool-registry import to the toolrunner's export
  → composite imports only WASI; exports runner

Browser path (build-time tooling, not a Rust dependency):
  @bytecodealliance/jco
    → transpiles dist/antikythera-sdk.wasm (the COMPOSITE) → npm/antikythera-sdk/component/
    → maps WASI imports (cli, io, clocks, filesystem, random) to local wasi-stubs/*.js

Legacy path (deprecated, crate-level compatibility only):
antikythera-wasm-bindgen
  → antikythera-sdk (features: component, no default-features)
    → antikythera-log (no wasm feature)

antikythera-sdk (features: wasm)
  → antikythera-log (features: wasm)
    → js-sys (browser WASM time via Date.now())
    → wasm-bindgen (JsValue type)
```

## Contracts per Target

### WASI Component — server (`wasm32-wasip1`)

**WIT file** (`wit/antikythera.wit`), canonical and wired to the build:

```wit
world antikythera-agent-sdk {
  import tool-registry;
  export runner;
}

world tool-registry-component {
  export tool-registry;
}
```

The `antikythera-agent-sdk` world exports the `runner` interface — 16 functions, every payload is a JSON string, errors are flattened to `string`; it imports the `tool-registry` interface provided by the composed `tool-registry-component` world:

| Function | Parameters | Return |
|----------|-----------|--------|
| `init` | `config-json: string` | `result<string, string>` |
| `prepare-user-turn` | `request-json: string` | `result<string, string>` |
| `commit-llm-response` | `prepared-turn-json: string, llm-response-json: string` | `result<string, string>` |
| `commit-llm-stream` | `prepared-turn-json: string` | `result<string, string>` |
| `process-llm-response-for-session` | `session-id: string, llm-response-json: string` | `result<string, string>` |
| `process-tool-result-for-session` | `session-id: string, tool-result-json: string` | `result<string, string>` |
| `append-llm-chunk` | `session-id: string, chunk: string, correlation-id: option<string>` | `result<bool, string>` |
| `drain-events` | `session-id: string` | `result<string, string>` |
| `get-state` | `session-id: string` | `result<string, string>` |
| `reset-session` | `session-id: string` | `result<bool, string>` |
| `sweep-idle-sessions` | `now-unix-ms: option<s64>` | `result<u32, string>` |
| `register-tools` | `tools-json: string` | `result<u32, string>` |
| `get-tools-prompt` | — | `result<string, string>` |
| `set-context-policy` | `policy-json: string` | `result<bool, string>` |
| `get-telemetry-snapshot` | `session-id: string` | `result<string, string>` |
| `get-slo-snapshot` | `session-id: string` | `result<string, string>` |

The host-facing interfaces (`host-imports`, `prompt-manager`, `mcp-client`, `ffi-server`) plus the shared `vocabulary` records are **preserved as vocabulary** in the WIT file but are **not yet imported by the world** — they are not part of the current component surface.

> **Decision: host-imports stays vocabulary.** The `host-imports` world remains a vocabulary contract in `wit/antikythera.wit`; it is **not activated** as an import of the composite. Rationale: activation would change the host contract from the current host-push architecture (host drives `runner` calls and feeds tool results via `process-tool-result-for-session`) to a host-pull model; that is a deliberately scoped future work item, not an execution decision in the current scope. No code change is required to keep it as vocabulary.

**Build** (composite deliverable; `task build` wraps all three steps, `task compose` wraps the last two):

```bash
cargo component build -p antikythera-sdk --release --target wasm32-wasip1 --no-default-features --features component
cargo component build -p antikythera-toolrunner --release --target wasm32-wasip1 --no-default-features --features component
cp target/wasm32-wasip1/release/antikythera_toolrunner.wasm target/wasm32-wasip1/release/antikythera-toolrunner.wasm
wasm-tools compose target/wasm32-wasip1/release/antikythera_sdk.wasm \
  -d target/wasm32-wasip1/release/antikythera-toolrunner.wasm \
  -o dist/antikythera-sdk.wasm
```

`wasm-tools compose` rejects crate names containing underscores, so the toolrunner artifact is first copied to its kebab-case name.

Canonical artifact: `dist/antikythera-sdk.wasm` — the **composite** (imports only WASI, exports `runner`). The standalone SDK artifact `target/wasm32-wasip1/release/antikythera_sdk.wasm` still imports `tool-registry` and is **not** a consumable deliverable.

**Server proof**: `examples/component-harness` is a wasmtime server binary that loads `dist/antikythera-sdk.wasm` and asserts the builtin tool `echo` executes inside the composite with `success=true` and no host round-trip (`cargo run -p component-harness`).

### Browser — same composite transpiled with jco

The browser path reuses the **composite WASI component** (`wasm32-wasip1`, feature `component` on both crates, composed with `wasm-tools compose`) and transpiles it to browser-safe ESM with `@bytecodealliance/jco`. Because the composite carries no non-WASI imports (the SDK's `tool-registry` import is satisfied by the embedded toolrunner), the transpiled module has no unmet imports and runs in Node/browser without a WASI host — transpiling the standalone SDK instead yields a module that fails at runtime.

**Output**: `npm/antikythera-sdk/component/` — ESM module exposing the `runner` namespace with 16 camelCase functions (all JSON strings; WIT `option<T>` renders as `T | undefined`; WIT errors surface as thrown JS errors):

| Function | Parameters | Return |
|----------|-----------|--------|
| `init` | `configJson: string` | `string` |
| `prepareUserTurn` | `requestJson: string` | `string` |
| `commitLlmResponse` | `preparedTurnJson: string, llmResponseJson: string` | `string` |
| `commitLlmStream` | `preparedTurnJson: string` | `string` |
| `processLlmResponseForSession` | `sessionId: string, llmResponseJson: string` | `string` |
| `processToolResultForSession` | `sessionId: string, toolResultJson: string` | `string` |
| `appendLlmChunk` | `sessionId: string, chunk: string, correlationId: string \| undefined` | `boolean` |
| `drainEvents` | `sessionId: string` | `string` |
| `getState` | `sessionId: string` | `string` |
| `resetSession` | `sessionId: string` | `boolean` |
| `sweepIdleSessions` | `nowUnixMs: bigint \| undefined` | `number` |
| `registerTools` | `toolsJson: string` | `number` |
| `getToolsPrompt` | — | `string` |
| `setContextPolicy` | `policyJson: string` | `boolean` |
| `getTelemetrySnapshot` | `sessionId: string` | `string` |
| `getSloSnapshot` | `sessionId: string` | `string` |

**WASI imports are mapped to local browser-safe stubs**: the component imports WASI preview1/wasi0.2 interfaces (`wasi:cli/*`, `wasi:io/*`, `wasi:clocks/*`, `wasi:filesystem/*`, `wasi:random/*`); jco maps each of them to `npm/antikythera-sdk/component/wasi-stubs/*.js` so the output runs in the browser without a Node/WASI host.

**Transpile command** (run after the composite build; the `transpile` task in `Taskfile.yml` depends on `compose`, so `task transpile` always consumes a fresh composite):

```bash
npx jco transpile dist/antikythera-sdk.wasm --out-dir npm/antikythera-sdk/component
```

Each WASI import is mapped to a local stub with `-M` flags. The 12 mappings (one per stub file) take the form:

```bash
-M wasi:cli/environment=./wasi-stubs/environment.js \
-M wasi:cli/exit=./wasi-stubs/exit.js \
-M wasi:cli/stderr=./wasi-stubs/stderr.js \
-M wasi:cli/stdin=./wasi-stubs/stdin.js \
-M wasi:cli/stdout=./wasi-stubs/stdout.js \
-M wasi:clocks/monotonic-clock=./wasi-stubs/monotonic-clock.js \
-M wasi:clocks/wall-clock=./wasi-stubs/wall-clock.js \
-M wasi:filesystem/preopens=./wasi-stubs/preopens.js \
-M wasi:filesystem/types=./wasi-stubs/types.js \
-M wasi:io/error=./wasi-stubs/error.js \
-M wasi:io/streams=./wasi-stubs/streams.js \
-M wasi:random/random=./wasi-stubs/random.js
```

Use the stub files as the source of truth for the current mapping set: if the component's WASI imports change, regenerate or adjust the `-M` flags to match the files present under `wasi-stubs/`.

### Sandbox (native)

**Exports expected from WASM module**: `antikythera_alloc`, `antikythera_run`, `memory`, `antikythera_dealloc`

**Host imports provided**: `call_llm_sync`

**Communication**: JSON over WASM memory (pointer+length packed as i64).

## WASM Time Abstraction (`wasm_compat`)

The `antikythera-log` crate provides `wasm_compat` module for platform-safe time:

| Target | Implementation | Mechanism |
|--------|---------------|-----------|
| Native (`not(wasm32)`) | `native.rs` | `chrono::Utc::now()` |
| WASI (`wasm32-wasip1`) — incl. the browser-via-jco component | `native.rs` | `chrono::Utc::now()` (WASI has system clock) |
| Browser without `wasm` feature | `native.rs` | `chrono::Utc::now()` (will panic at runtime) |
| `wasm32-unknown-unknown` + `wasm` feature (**legacy**) | `browser.rs` | `js_sys::Date::now()` |

**Functions**:
- `now_unix_ms() -> i64` — Unix timestamp in milliseconds
- `now_rfc3339() -> String` — RFC 3339 formatted timestamp
- `now_timestamp_nanos() -> i64` — Unix timestamp in nanoseconds

**Why this exists**: `chrono::Utc::now()` calls `SystemTime::now()` which panics in `wasm32-unknown-unknown`. The `wasm` feature gates the browser-safe implementation using JavaScript's `Date.now()`. The WASI component path does **not** need this fallback because WASI provides a system clock; the legacy `wasm` feature remains only for crate-level compatibility.

## Common Pitfalls

### 1. jco has no `--dts` flag

**Symptom**: `jco transpile ... --dts` fails with an unknown-option error.

**Fix**: In jco, TypeScript declarations are emitted by default; pass `--no-typescript` only if you want to suppress them. Do not pass `--dts`.

### 2. Top-level await requires an ES2022 build target

**Symptom**: Bundlers (e.g. Vite) fail to build the transpiled ESM module because it uses top-level await.

**Fix**: Set the build target to `es2022` (the web example's `vite.config.ts` does this: `build: { target: 'es2022' }`).

### 3. `preview2-shim` vs `--map` for WASI imports

**Symptom**: The transpiled module depends on `@bytecodealliance/preview2-shim` (jco's default WASI rewrite), which is not browser-safe, or the stub imports point at files that do not exist.

**Fix**: Map every WASI import to a local stub under `wasi-stubs/` with `-M` flags (see the browser contract above). Use `--no-wasi-shim` if jco's automatic preview2-shim rewriting gets in the way.

### 4. `init` returns a plain `session_id`, not JSON

**Symptom**: Callers treat the `init` return value as a JSON payload.

**Fix**: `init` returns the raw session id string (e.g. `session-...`). Treat it as an opaque string; do not `JSON.parse` it.

### 5. `chrono::Utc::now()` panics in legacy browser WASM

**Symptom**: Runtime panic when calling `chrono::Utc::now()` in `wasm32-unknown-unknown`.

**Fix**: This only affects the **legacy** `wasm` feature path. The WASI component (`wasm32-wasip1`) has a system clock, so `antikythera_log::wasm_compat::now_unix_ms()` works without the `wasm` feature. On the legacy path, ensure the `wasm` feature is enabled.

### 6. Wrong feature flag for target

**Symptom**: Compilation error or runtime panic.

**Checklist**:
- `wasm32-wasip1` → use `component` feature
- browser (current) → build the `component` for `wasm32-wasip1`, then `jco transpile`
- `wasm32-unknown-unknown` → `wasm` feature (**legacy** only)
- native → use `sdk-core` or `full` features

### 7. FSM parity drift

**Symptom**: WASM-side `AgentFsmState` diverges from core's `AgentFsmState`.

**Mitigation**: Run `fsm_parity_tests` — golden-file tests verify cross-crate FSM state parity.

### 8. Contract drift

**Symptom**: WIT signatures or JSON payload shapes change without updating consumers.

**Mitigation**: Run `cargo test -p antikythera-tests --test compatibility_tests` — `browser_type_signatures_match_golden`, `payload_contract_shapes_match_golden`, and the runner namespace re-export test detect breaking changes against the transpiled `antikythera-agent-sdk-runner.d.ts` and the golden files.

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
2. **WASM compile check** — `cargo check` for `wasm32-wasip1` (feature `component`); the legacy `wasm32-unknown-unknown` + `wasm` feature check is retained for the deprecated path
3. **WIT conformance validation** — `cargo run -p build-scripts --release -- validate`
4. **Composite build** — build SDK component (`cargo component build -p antikythera-sdk --release --target wasm32-wasip1 --no-default-features --features component`), build toolrunner component (`-p antikythera-toolrunner`), then `wasm-tools compose` both into `dist/antikythera-sdk.wasm` (the kebab-case copy of the toolrunner is required — compose rejects underscore names); only then transpile with jco and smoke-test in Node
5. **Contract tests** — `cargo test -p antikythera-tests --test compatibility_tests` (runner signatures vs golden, payload shapes vs golden, runner namespace re-export)
6. **Documentation build** — mdBook build
