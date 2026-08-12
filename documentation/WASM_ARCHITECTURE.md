# WASM Architecture

This document describes the WASM integration paths in the Antikythera Agent SDK, their target platforms, feature flags, contracts, and common pitfalls.

Currently, there are **two active WASM paths** (WASI Component for server, and the same component transpiled by `jco` for the browser), plus a **native sandbox** runner. The former `wasm-bindgen` browser path (`wasm32-unknown-unknown` + `wasm` feature) is **deprecated** and retained only for crate-level compatibility.

The WASI component deliverable is **composite** (three members): `dist/antikythera-sdk.wasm` is produced by `wasm-tools compose`, wiring the `antikythera-sdk` component (exports `runner`, imports `tool-registry` and `logic-hooks`) to the `antikythera-toolrunner` component (exports `tool-registry`, builtin tool execution) and the `antikythera-default-hooks` component (exports `logic-hooks`, no-op passthrough). The standalone SDK component is an **intermediate artifact only** — it carries unmet `tool-registry` and `logic-hooks` imports and must never be transpiled or embedded directly.

## WASM Paths

| Path | Target | Feature Flag | Build Command | Contract | When to Use |
|------|--------|-------------|---------------|----------|-------------|
| **WASI Component (server)** | `wasm32-wasip1` | `component` (SDK, toolrunner, default-hooks) | `task build` = `cargo component build -p antikythera-sdk` + `-p antikythera-toolrunner` + `-p antikythera-default-hooks` + `wasm-tools compose` | `wit/antikythera.wit` (worlds `antikythera-agent-sdk` + `tool-registry-component` + `logic-hooks-component`); composite exports `runner`, imports only WASI | Server-side: host embeds wasmtime and calls exports via FFI |
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
  → imports antikythera:agent-sdk/tool-registry and
    antikythera:agent-sdk/logic-hooks (world antikythera-agent-sdk)

antikythera-toolrunner (features: component)
  → wit-bindgen (WIT bindings generator for the component world)
  → exports antikythera:agent-sdk/tool-registry (world tool-registry-component,
    builtin tool execution — stateless, no host round-trip)

antikythera-default-hooks (features: component)
  → wit-bindgen (WIT bindings generator for the component world)
  → exports antikythera:agent-sdk/logic-hooks (world logic-hooks-component,
    no-op passthrough — every hook returns {"passthrough": true})

Composite deliverable, build-time tooling:
  wasm-tools compose target/.../antikythera_sdk.wasm \
    -d target/.../antikythera-toolrunner.wasm \
    -d target/.../antikythera-default-hooks.wasm \
    -o dist/antikythera-sdk.wasm
  → wires the SDK's tool-registry import to the toolrunner's export
  → wires the SDK's logic-hooks import to the default-hooks' export
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

The `antikythera-agent-sdk` world exports the `runner` interface — 16 functions, every payload is a JSON string, errors are flattened to `string`; it imports the `tool-registry` interface provided by the composed `tool-registry-component` world and the `logic-hooks` interface provided by the composed `logic-hooks-component` world:

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

The host-facing interfaces (`prompt-manager`, `mcp-client`, `ffi-server`) plus the shared `vocabulary` records are **preserved as vocabulary** in the WIT file — they are not part of any world surface. `host-imports` is the exception: it is **activated** as an import of the `logic-core-component` world (drop-in logic cores) — see [Host-imports (activated for drop-in logic cores)](#host-imports-activated-for-drop-in-logic-cores) below.

> **Decision: host-imports activated for drop-in logic cores.** The `logic-core-component` world imports `host-imports`, and a logic core that references it gets a host-pull escape hatch: the component calls the host (LLM, state, tools, logging) instead of the host calling it. The SDK composite does **not** import `host-imports` — it keeps the host-push contract (host drives `runner` calls and feeds tool results via `process-tool-result-for-session`). The two models coexist — host-push for the SDK composite, host-pull for logic cores that choose to import `host-imports` (**hybrid model**). A host MUST implement `host-imports` behind permission gates before loading any component that imports it; without permission the component is rejected (fail-closed).

**Build** (composite deliverable; `task build` runs the full composite build, `task compose` wraps the compose step):

```bash
cargo component build -p antikythera-sdk --release --target wasm32-wasip1 --no-default-features --features component
cargo component build -p antikythera-toolrunner --release --target wasm32-wasip1 --no-default-features --features component
cargo component build -p antikythera-default-hooks --release --target wasm32-wasip1 --no-default-features --features component
cp target/wasm32-wasip1/release/antikythera_toolrunner.wasm target/wasm32-wasip1/release/antikythera-toolrunner.wasm
cp target/wasm32-wasip1/release/antikythera_default_hooks.wasm target/wasm32-wasip1/release/antikythera-default-hooks.wasm
wasm-tools compose target/wasm32-wasip1/release/antikythera_sdk.wasm \
  -d target/wasm32-wasip1/release/antikythera-toolrunner.wasm \
  -d target/wasm32-wasip1/release/antikythera-default-hooks.wasm \
  -o dist/antikythera-sdk.wasm
```

`wasm-tools compose` rejects crate names containing underscores, so the toolrunner and default-hooks artifacts are first copied to their kebab-case names.

Canonical artifact: `dist/antikythera-sdk.wasm` — the **composite** (imports only WASI, exports `runner`). The standalone SDK artifact `target/wasm32-wasip1/release/antikythera_sdk.wasm` still imports `tool-registry` and `logic-hooks` and is **not** a consumable deliverable.

**Server proof**: `examples/component-harness` is a wasmtime server binary that loads a component path (default `dist/antikythera-sdk.wasm`) and asserts the builtin tool `echo` executes inside the composite with `success=true` and no host round-trip. The final probe is generic: with `--expect=final --expect-content=<string>` it asserts the commit envelope carries `action=final` with the given content and no tool executes — covering a host-authored `decide-action` override (`hook-forced-final`) or a drop-in logic core's deterministic commit (`echo-agent-done`); `--expect=notimpl` probes logic-core template holes and asserts the structured `{"error":"not implemented",...}` error (`cargo run -p component-harness -- <component-path> --expect=default|final|notimpl --expect-content=<string>`).

### Browser — same composite transpiled with jco

The browser path reuses the **composite WASI component** (`wasm32-wasip1`, feature `component` on all three crates, composed with `wasm-tools compose`) and transpiles it to browser-safe ESM with `@bytecodealliance/jco`. Because the composite carries no non-WASI imports (the SDK's `tool-registry` import is satisfied by the embedded toolrunner and its `logic-hooks` import by the embedded default-hooks passthrough), the transpiled module has no unmet imports and runs in Node/browser without a WASI host — transpiling the standalone SDK instead yields a module that fails at runtime.

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

### Logic hooks (plug-and-play host logic)

The composite's third member provides the `logic-hooks` interface. The SDK component imports `antikythera:agent-sdk/logic-hooks`; a composed provider exports it from the `logic-hooks-component` world. The default deliverable embeds `plugin/antikythera-default-hooks`, a no-op passthrough provider whose every hook returns `{"passthrough": true}` — so the default composite behaves exactly like the SDK alone. A host-authored hooks component can be composed in its place to customize pipeline decisions without modifying the SDK.

The SDK runner calls three hooks at fixed pipeline points:

| Hook | Pipeline point | Inputs |
|------|----------------|--------|
| `prepare-turn` | start of `prepare-user-turn`, after the SDK has built its default prepared turn | `request-json`, `session-state-json` |
| `decide-action` | LLM response commit path (`process-llm-response-for-session`), before default action derivation | `session-state-json`, `llm-response-json` |
| `handle-tool-result` | tool result reporting (`process-tool-result-for-session`), before default processing | `session-state-json`, `tool-result-json` |

Each hook is a pure decision function: it receives JSON inputs and returns a JSON decision (`result<string, string>`). The return semantics are uniform across all three points:

- Ok `{"passthrough": true}` — the single-key passthrough signal: keep the SDK default behavior at this point.
- Ok \<any other JSON object\> — override: use this value in place of the SDK default decision (per-hook merge semantics are documented in `wit/antikythera.wit`).
- Err(message) — the hook failed; the SDK aborts the operation and surfaces the error. A failing hook never falls back to passthrough.
- A return that is not parseable JSON or not a JSON object is treated as hook failure (Err-path semantics) — fail-closed.

Hooks are stateless. Session state is owned by the SDK: it is passed in as `session-state-json` on every call, hooks must not persist or mutate it, and any state a hook writes is discarded.

> **Host note — terminal `decide-action` overrides keep the default envelope.** The SDK merges a `decide-action` override over the default commit-result envelope (`logic_hooks.rs`, `apply_decide_action_override`), so a partial override that forces a terminal action (for example `{"action":"final","content":"hook-forced-final"}`) retains the bookkeeping fields derived from the default decision: `tool_name` and `fsm_state` keep their default values, and the session state stays `tool_requested` (default derivation already emitted the `tool_requested` event before the hook is consulted). Hosts MUST NOT branch on `fsm_state` or `tool_name` after an override — the `action` in the envelope is authoritative. Because the session remains `tool_requested`, the host may still call `process-tool-result-for-session` after a terminal action; the tool-result path accepts it (the runner's FSM guard re-establishes `tool_requested` if needed). The same merge semantics are documented in the WIT contract (`wit/antikythera.wit`, `interface logic-hooks`, `decide-action`).

**Default vs custom composition.** `task compose` produces the three-way default composite (SDK + toolrunner + default-hooks). For host-authored hooks, build a `logic-hooks-component` and compose it with the same `-d` wiring used for the toolrunner (`wasm-tools compose ... -d <hooks>.wasm`; `task compose-hooks-custom` wraps the custom flow). The template `examples/logic-hooks-template/` is the starting point — edit only the three `custom_*` functions in `src/lib.rs` (`custom_prepare_turn`, `custom_decide_action`, `custom_handle_tool_result`): `None` means passthrough, `Some(json)` means override. `examples/logic-hooks-example/` is a filled-in probe whose `decide-action` always returns `{"action":"final","content":"hook-forced-final"}`, forcing a terminal action regardless of the committed LLM response.

**Host-push architecture is unchanged for the SDK composite.** Logic hooks are decision points inside the existing host-push model: the host still drives `runner` calls and feeds tool results via `process-tool-result-for-session`. Composing custom hooks does not switch the model to host-pull. Host-pull is available only to drop-in logic cores that import `host-imports` (see below); the SDK composite never imports it. See [`BUILD.md`](BUILD.md) — Host-authored logic hooks for the build and verification flow.

### Drop-in logic core (swap-able runner)

The deepest customization is replacing the runner itself. The WIT world `logic-core-component` (`import host-imports; import tool-registry; export runner;`) is the drop-in contract: a host-authored component that exports the same `runner` interface listed above — the same 16 functions with the same JSON-string semantics — loads wherever the SDK composite loads, with zero host code changes. Host code calls the same API whether the loaded component is the SDK composite (wasmtime server, jco client) or a custom logic core; the same host script runs both.

The world's imports are optional declarations, not requirements:

- `host-imports` — a logic core that wants to call host LLM, tool, log, or state services may import this interface and use it; a component that never references it remains valid.
- `tool-registry` — a logic core that wants to reuse the stateless toolrunner catalog and executor (`list-tools-json`, `validate-tool-call`, `execute-builtin`) may import this interface.

An import that nothing in the component references is pruned by the component encoder, so a self-contained logic core ends up importing nothing but WASI. The template and the deterministic example both import only WASI; the host-imports example (`examples/logic-core-host-example/`) actually calls the host helpers, so its artifact carries `import antikythera:agent-sdk/host-imports@1.0.0;` — the activation proof. All logic cores are consumed directly as standalone artifacts — never composed into `dist/` (unlike the hooks member of the composite).

`examples/logic-core-template/` is the starting point. It ships 14 `custom_*` hooks in `src/lib.rs` — one per runner function except `get-state` and `reset-session`, which are fixed in-memory store plumbing (`init`, `get-state`, `reset-session` work out of the box). Every hook defaults to `None`; the adapter in `component.rs` maps `None` to the template default, and for every runner function without a default it returns the structured error `{"error":"not implemented","function":"<kebab-case-name>"}` so host code can detect a template hole deterministically.

`examples/logic-core-example/` is a filled-in probe proving the swap: a deterministic "echo-agent" whose `prepare-user-turn` builds the standard prepared-turn envelope and whose `commit-llm-response` always commits `{"action":"final","content":"echo-agent-done",...}` — no LLM, no host imports.

`examples/logic-core-host-example/` is the third sibling: a full custom loop that reaches the host for the LLM and for tool execution through the `host-imports` escape hatch (see the next section) instead of computing everything deterministically inside the component. Its session state lives in the host (`save-state` / `load-state`), and its `get-state` / `reset-session` report `Session not found` — the component is stateless between runner calls.

**Server proof**: the harness final probe runs the standard init → prepare-user-turn → commit-llm-response → drain-events flow against the logic-core artifact and asserts the commit envelope carries `action=final` and `content=echo-agent-done` with no tool result:

```bash
cargo run -p component-harness --release -- <logic-core>.wasm --expect=final --expect-content=echo-agent-done
```

`--expect=notimpl` calls template-hole functions (`get-tools-prompt`, `sweep-idle-sessions`) and asserts the structured not-implemented error. The same final probe covers a logic-hooks override (`--expect-content=hook-forced-final`), which is how the harness stays one generic flow for both customization paths.

**Client proof**: because a self-contained logic core imports only WASI, it transpiles with jco the same way the composite does (same WASI stub mapping) and runs in the Node probe and the Vite bundle.

**Difference from logic hooks.** Logic hooks are customization points *inside* the SDK runner: three stateless decision points (`prepare-turn`, `decide-action`, `handle-tool-result`) that passthrough, override, or abort while the SDK keeps owning session state, tool execution, and the FSM. A drop-in logic core *replaces the whole runner*: the host authors the 16-function `runner` implementation itself and owns every pipeline decision. Together with data-driven tool registration via `register-tools` (the host supplies tool definitions as JSON at runtime), the runner offers three customization paths at different layers: customize the tool surface as data, override decisions at fixed points via composed hooks, or replace the runner with a drop-in logic core.

### Host-imports (activated for drop-in logic cores)

`host-imports` is the escape hatch that turns a logic core from host-push into host-pull: the component calls the host instead of being called by it. It is declared on the `logic-core-component` world and is **activated** whenever a logic core's code actually references it — the component encoder prunes the import otherwise, so the SDK composite (which never references it) stays host-push.

**Interface** (`wit/antikythera.wit`, `interface host-imports`; the import name `antikythera:agent-sdk/host-imports@1.0.0` is the contract identity — it is what the host linker registers and what jco maps). Five functions, all JSON-string semantics matching the rest of the surface:

| Function | Purpose | Return |
|----------|---------|--------|
| `call-llm` | request an LLM completion from the host | `result<llm-response, string>` |
| `save-state` | persist session state with the host | `result<_, string>` |
| `load-state` | load state previously saved by the host | `result<option<string>, string>` |
| `emit-tool-call` | ask the host to execute a tool | `result<tool-execution-result, string>` |
| `log-message` | send a log line to the host | — |

**Helper template.** `examples/logic-core-template/` (and its filled-in sibling `examples/logic-core-host-example/`) ships five Rust helpers in `src/lib.rs`, gated by `#[cfg(feature = "component")]`: `host_call_llm`, `host_save_state`, `host_load_state`, `host_emit_tool_call`, and `host_log`. They bind the generated `host-imports` bindings and translate the WIT records to and from the JSON-string convention; the native build compiles them as stubs, so the loop only runs inside the component.

**Host obligation — permission gates are mandatory.** A host that wires `host-imports` into its linker is granting the component an exit to the outside world; it MUST implement the five functions behind permission gates. The reference implementation is `examples/component-harness` (wasmtime server): `Harness::add_to_linker` registers the import instance `antikythera:agent-sdk/host-imports@1.0.0`, and every gate fails explicitly with a `permission:` message — there is no silent degradation:

- `call-llm` — **quota**: at most 3 calls per instance; beyond that `Err("permission: llm quota exceeded")`.
- `emit-tool-call` — **allowlist**: only the builtin `echo` executes; anything else `Err("permission: tool '<name>' not in allowlist")`.
- `save-state` / `load-state` — **bounded storage**: files live under a fixed storage root (`<storage-dir>/<context-id>.json`), and the context-id is validated before any filesystem access; traversal ids are rejected `Err("permission: invalid context id")`.
- `log-message` — passthrough to host stderr.

**Server proof.** The harness runs four probes against the host-imports artifact (`target/wasm32-wasip1/release/logic_core_host_example.wasm`):

```bash
cargo run -p component-harness --release -- \
  target/wasm32-wasip1/release/logic_core_host_example.wasm \
  --probe=full-loop|quota|allowlist|storage
```

`full-loop` proves `call-llm` is reached (the committed content is the host stub's `stub-llm-response`, not a guest constant) and `emit-tool-call` executes the allowlisted `echo`; `quota`, `allowlist`, and `storage` assert each gate rejects with the exact `permission:` message.

**Client wiring.** A logic core that imports `host-imports` transpiles with jco like any component; the versioned import is mapped to a JS module that implements the same gates — see [`BUILD.md`](BUILD.md) — Host-imports wiring for the `-M` flag, the import object, and the plain-string error rule.

**State ownership.** With `host-imports`, session state may live in the host: `save-state` / `load-state` persist and restore it, and the component stays stateless between runner calls (`examples/logic-core-host-example/` re-reads state at every hook entry and re-persists at exit). The host decides how much of the session the component may touch — the same permission gates apply.

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
4. **Composite build** — build SDK component (`cargo component build -p antikythera-sdk --release --target wasm32-wasip1 --no-default-features --features component`), build toolrunner component (`-p antikythera-toolrunner`), build default-hooks component (`-p antikythera-default-hooks`), then `wasm-tools compose` the three into `dist/antikythera-sdk.wasm` (the kebab-case copies of the component artifacts are required — compose rejects underscore names); only then transpile with jco and smoke-test in Node
5. **Contract tests** — `cargo test -p antikythera-tests --test compatibility_tests` (runner signatures vs golden, payload shapes vs golden, runner namespace re-export)
6. **Documentation build** — mdBook build
