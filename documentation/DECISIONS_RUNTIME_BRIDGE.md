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
- One active core instance (client OR server); the side is selected when the
  runtime is created — a running session is not migrated between sides. State
  lives where the core runs. Resources on both sides remain reachable.
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

## Python Bridge Server + jco Delivery (D1–D6)

Locked for the delivery that adds a drop-in Python bridge server to the
Runtime Bridge and ships the jco-transpiled bundle to Python users. The
D-numbers in this section are delivery-local labels and are unrelated to
the D1/D2 labels inside decision (d).

### Decision D1 — jco bundle delivery to Python users

**Chosen (defensive): ship the jco bundle inside the Python wheel as
package data; do not transpile at runtime.**

- The transpiled ESM bundle (entry `antikythera-sdk.js`, plus its
  `runtime-hooks` import stub) is produced once at build time and shipped
  as package data in the wheel, next to the composite `.wasm` already
  covered by `[tool.setuptools.package-data]` (`*.wasm`).
- The wheel is then self-contained: a Python user can serve the bundle to
  a browser without Node, jco, or any JavaScript toolchain at runtime.
- **Rejected: transpile at install/runtime** — forces Node and jco onto
  every Python user, including those who never run the browser path, and
  moves a build-time concern into install/run.
- **Rejected: serve the raw composite `.wasm`** — browsers cannot execute
  a composite WASI component directly; the component must be
  jco-transpiled before the browser can import it, so the transpiled JS
  bundle is the unit that ships.

Downstream impact: the wheel gains the jco output as package data and
grows by the bundle size; the build pipeline gains a jco transpile step
before packaging; wheel and npm package must be released in lockstep so
the manifest version always matches the shipped bundle.

### Decision D2 — Python server is a drop-in wire-protocol peer

**Chosen (defensive): the Python server is a drop-in peer of the
reference server; the wire protocol surface is UNCHANGED.**

- The Python server conforms to `contracts/shared/wire_protocol.golden.json`
  and [WIRE_PROTOCOL.md](WIRE_PROTOCOL.md); clients cannot tell which
  implementation answered the request.
- **Rejected: a new or Python-specific protocol** — would fork the shared
  contract, force clients to branch per implementation, and invalidate
  the golden file as the single source of truth.

Downstream impact: any wire-protocol change must be registered here and
mirrored in BOTH implementations; the Python server must pass the same
golden contract tests as the reference server; client-side code needs no
Python-specific branches.

### Decision D3 — Transport default and replaceability

**Chosen (defensive): stdlib `ThreadingHTTPServer` as the default
transport, behind a transport port/interface.**

- Zero-dependency default: `threading`/`http.server` ship with the
  stdlib, so the base Python install stays dependency-free.
- The transport is isolated behind an interface so an asyncio/aiohttp
  implementation can replace it without touching any other unit.
- Validity envelope: tens of concurrent SSE clients —
  `ThreadingHTTPServer` holds one thread per long-lived SSE connection;
  at or above that scale, swap the transport behind the interface
  instead of tuning the default.
- **Rejected: aiohttp as the default transport** — adds a mandatory
  dependency for a scale this delivery does not target.

Downstream impact: SSE handling must be thread-safe (one long-lived
connection per handler thread); the transport interface and its
replacement contract are defined and covered by tests so the swap is
drop-in.

### Decision D4 — jco delivery as a recorded protocol extension

**Chosen (defensive): additive protocol extension; old shapes unchanged.**

- `GET /antikythera/v1/component/manifest` →
  `{"base": "/antikythera/v1/component/", "entry": "antikythera-sdk.js",
  "version": "<sdk-version>"}`.
- `GET /antikythera/v1/component/{path}` — static file serving; MIME
  types: `.js` = `text/javascript`, `.wasm` = `application/wasm`.
- The manifest response shape is added to
  `contracts/shared/wire_protocol.golden.json` as a new entry; the two
  endpoints are added to [WIRE_PROTOCOL.md](WIRE_PROTOCOL.md).
- The extension is additive: it adds endpoints without altering any
  existing endpoint or record shape.
- **Rejected: hardcoding the directory layout in the client** — clients
  would break on any future layout change; the manifest makes the layout
  a server responsibility that the client resolves at runtime.

Downstream impact: `entry` names the same jco output the npm package
already ships (`npm/antikythera-sdk/component/antikythera-sdk.js`); the
server must serve the bundle with the registered MIME types; clients
resolve bundle paths from the manifest instead of assuming a layout;
manifest version vs client version skew becomes a client-visible signal.

### Decision D5 — Minimal JS client extension

**Chosen (defensive): additive options on `createAgentRuntime`; default
behavior unchanged.**

- `componentBase?: string` — absolute URL of the bundle directory; lets
  the client load the jco bundle from the Python server (resolved from
  the manifest, or set directly).
- `runner?: RunnerNamespace` — direct injection of a runner namespace,
  for consumers that already hold a runner.
- Defaults remain the bundled path, so existing consumers are untouched
  (backward compatible).
- **Rejected: changing the default behavior** — any default change would
  break existing consumers and violate the drop-in peer property (D2).

Downstream impact: the JS client type declarations gain the two optional
fields; the client keeps a single code path with the bundled path as the
fallback; no existing option changes meaning.

### Decision D6 — wasmtime is optional, not mandatory

**Chosen (defensive): core@client does NOT require wasmtime; core@server
does (optional `[wasm]` extra).**

- core@client mode is static files + HTTP only; it never instantiates a
  native runtime, so wasmtime is not needed.
- core@server mode runs the composite on wasmtime, which remains an
  optional dependency via the existing `[wasm]` extra
  (`wasmtime>=42.0`).
- Base install (`pip install antikythera-agent`) stays native-dependency
  free; the `[wasm]` extra remains the single place where the wasmtime
  version is pinned.
- **Rejected: wasmtime as a mandatory dependency** — forces a heavy
  native dependency on every Python user, including those who only run
  the client path.

Downstream impact: the server-core path is the only code path that
touches wasmtime; server-core users must install
`pip install antikythera-agent[wasm]`; the base wheel keeps its
zero-dependency install.

### Decision D7 — Python bridge = server-side peer only (R7)

**Chosen (defensive): the Python package is a server-side wire peer
only; it ships no SSE-client peer.**

- Context: the Python delivery already serves the LLM proxy, the SSE
  control channel, and the static jco bundle, with the optional
  `core@server` loop via wasmtime (D6). That leaves one side of the
  wire unclaimed: whether the Python package should also embed the
  SSE-client peer — the host runtime that connects to a server.
- Decision: it does not. The client-side runtime role stays owned by
  the JavaScript package (`createAgentRuntime` from
  `antikythera-agent/runtime`); Python owns the server side of the
  wire only.
- Consequences: the wire is asymmetric by design — Python is the
  server peer, JS is the client peer; there is no SSE-client
  reimplementation to build or maintain in Python; client-side protocol
  evolution does not force changes in the Python package beyond shared
  golden-contract parity (D2); consumers needing a programmatic client
  compose the JS runtime.
- This decision is permanent until changed in this register.

Downstream impact: the Python wheel never gains an HTTP/SSE client
dependency surface; protocol work stays two-sided but role-separated
(server parity in Python, client behavior in JS); reversing this
decision requires a new registered entry here plus a deprecation path
for the JS-only client assumption.
