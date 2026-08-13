# Decision Register — Runtime Bridge (Client–Server Connectivity)

This document registers the four design decision points for the Runtime Bridge
feature (automatic bidirectional client–server connectivity for the WASM core)
and the defensive choices that were locked before execution. Every option is
recorded, including rejected ones, with the downstream impact of the chosen
option. This is the source of truth for why the contracts look the way they
do; do not re-litigate a registered decision without updating this file.

Locked context (from the audit, findings T1–T5, R1–R6, K1–K2):

- The core is a composite WASI component (`dist/antikythera-sdk.wasm`) that
  runs on the server (wasmtime) or the client (jco-transpiled ESM). It
  currently has no host round-trip channel for non-builtin tools and no
  transport between client and server.
- Transport is HTTP + SSE. SSE is server→client only, so server-initiated
  tool requests to the browser travel over an SSE control channel with
  client POST-back.
- One active core instance (client OR server); state lives where the core
  runs. Resources on both sides remain reachable.
- Scope = model interaction: LLM routing, tools, hooks. Storage and LLM
  providers stay server-side; ALL LLM calls (including local-dev Ollama)
  proxy through the server.
- Security = default-deny permission gates, policy configurable by the SDK
  developer. Denials MUST surface as `permission:` errors (repo invariant).
- Tools are locked to one side (client-only or server-only). The registry is
  a union; cross-side name collisions are rejected at registration.
- Drop-in logic cores are in scope, including two-way remote calls and
  third-party (MCP) tool calls. Routing destinations: local / remote / mcp.

## Decision (a) — Shape of the WIT extension for runtime hooks

**Chosen (defensive): A1 + A1a.**

- **A1**: a new imported interface `runtime-hooks` with three functions whose
  signatures are IDENTICAL to `logic-hooks` (`prepare-turn`,
  `decide-action`, `handle-tool-result`, each
  `(string, string) -> result<string, string>`). Import name
  `antikythera:agent-sdk/runtime-hooks@1.0.0`. Added to the
  `antikythera-agent-sdk` world. The exported `runner` surface stays at 16
  functions. A config flag `runtime_hooks_enabled` (default true) controls
  whether the runtime provider is consulted.
- **A1a (precedence)**: the composed `logic-hooks` provider is consulted
  FIRST; `runtime-hooks` is invoked only when the composed provider returns
  the passthrough signal. Preserves existing consumer behavior.
- **A2** (extend `host-imports` and import it from the SDK world): rejected —
  mixes the host-push/host-pull models; `host-imports` is designed for
  drop-in logic cores and carries permission-gate obligations that do not
  belong on the default composite path.
- **A3** (new `runner` exports for hook callback registration): rejected
  technically — hook decisions are consumed synchronously inside the runner
  at pipeline points; a WASM component cannot call the host without an
  import, so an export-based callback channel cannot deliver the decision
  mid-call.

Downstream impact: the composite loses the "imports only WASI" property; it
now imports exactly one non-WASI interface (`runtime-hooks`). All consumers
must be updated in the same delivery: jco transpile gains one `-M` mapping,
the wasmtime harness wires the import in its linker, golden contract tests
assert exactly one non-WASI import.

## Decision (b) — Shape of the wire protocol (HTTP + SSE)

**Chosen (defensive):**

- LLM streaming travels as `llm-token` events on the SSE control channel
  (not a separate chunked-stream endpoint). The client feeds each token into
  `append-llm-chunk`.
- Endpoints:
  - `POST /antikythera/v1/llm/call` — body is the `llm-request` record
    shape, response is the `llm-response` record shape. All LLM calls
    proxy through the server (R6).
  - `POST /antikythera/v1/tools/execute` — body is the `tool-call-event`
    shape, response is the `tool-execution-result` shape. Used for
    `server`- and `mcp`-owned tools when the core runs on the client.
  - `GET /antikythera/v1/tools` — registry pull for discovery (C1).
  - `GET /antikythera/v1/events?client_id=...&session_id=...` — SSE stream.
  - `POST /antikythera/v1/events/{correlation-id}/response` — POST-back.
- SSE connection lifecycle (per-session vs per-client) is developer
  flexibility: the protocol supports both, the framework does not choose.

## Decision (c) — Tool discovery across sides (registry union sync)

**Chosen (defensive): C1 pull-on-demand.**

- The loop owner pulls the peer registry via `GET /tools` at session init and
  on explicit re-sync. The union is computed locally and pushed to the
  runner in a single `register-tools` call (because `register-tools` replaces
  the entire registry).
- **C2** (push on registration via `registry-sync` events): recorded, not
  chosen — adds an extra event family and ordering complexity for no benefit
  at the current scale.
- **C3** (static manifest): recorded, not chosen — no runtime sync means
  client tool sets cannot change without redeploying the peer.

## Decision (d) — Session handling when the core runs on the client

**Chosen (defensive): D2 — deferred.**

- Session state stays on the core side (decision: one active core instance).
  A browser reload loses the session; this is documented behavior.
- State endpoints (`POST|GET /antikythera/v1/state/{context-id}`) are
  RESERVED as "future" in the wire protocol and are NOT exposed by this
  delivery. Implementing them would require a 17th runner export
  (`restore-session`) which would cascade through every golden contract —
  deferred to a follow-up delivery.
- **D1** (in scope): recorded, rejected for this delivery for the reason
  above.

## Supporting decisions (locked, not re-litigated)

- All LLM calls proxy through the server, including local-dev Ollama (R6).
- Permission gates are default-deny; every denial surfaces as a
  `permission:` error (R4). Policy is configurable by the SDK developer.
- Tools are locked to one side; the registry is a union; name collisions are
  rejected explicitly at registration (R5).
- MCP is a third routing destination (`mcp`), always executed server-side
  (stdio transport is unavailable in the browser) (K2).
- The tool loop lives in the host runtime, not in the runner
  (`auto_execute_tools=false` pattern) (K1).
