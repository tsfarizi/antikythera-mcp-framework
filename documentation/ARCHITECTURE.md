# Architecture

This document gives a current high-level view of how the framework crates interact.

## System view

```mermaid
flowchart TD
    USER[Host application]
    SDK[antikythera-sdk]
    CORE[antikythera-core]
    DOMAIN[antikythera-domain]
    PORTS[antikythera-ports]
    CONFIG[antikythera-config]
    SESSION[antikythera-session]
    STORAGE[antikythera-storage]
    LOG[antikythera-log]
    RESILIENCE[antikythera-resilience]
    TOOLING[antikythera-tooling]
    MCP[MCP servers]
    LLM[LLM providers]

    USER --> SDK
    SDK --> CORE
    SDK --> SESSION
    SDK --> LOG
    CORE --> DOMAIN
    CORE --> PORTS
    CORE --> CONFIG
    CORE --> LOG
    CORE --> SESSION
    CORE --> RESILIENCE
    CORE --> TOOLING
    CORE --> MCP
    CORE --> LLM
    STORAGE --> SESSION
    STORAGE --> LOG
    LOG --> PORTS
```

## Core Principles

- **Single Source of Truth for Session:** `antikythera-session` owns the conversational data model (`Message`, `MessageRole`, `MessagePart`) and provides the thread-safe `SessionManager`. Both `CORE` (for actual context injection) and host applications utilize this unified model.
- **Pluggable Persistence:** `antikythera-storage` provides session persistence with pluggable backends (filesystem, MongoDB, PostgreSQL), in-memory caching with TTL/LRU eviction, and backup coordination. Host applications integrate storage via the `--storage` flag or direct API.
- **Stateless Tooling:** `CORE` orchestrates LLM dispatch, agent loops, and MCP tools, delegating long-term conversational memory to `SESSION` and `STORAGE`.
- **FFI & Portability:** `SDK` exposes `SESSION` and `LOG` components over safe FFI boundaries, allowing host languages (e.g. Node.js, Python) to import/export chat histories easily using the `Postcard` binary format.

## Request flow

```mermaid
sequenceDiagram
    participant User
    participant Host as Host application (SDK)
    participant Core as antikythera-core
    participant Session as antikythera-session
    participant Provider as LLM provider
    participant Server as MCP server

    User->>Host: Send prompt or task
    Host->>Core: Build request
    Core->>Session: Load previous history & metadata
    Core->>Provider: Generate response
    Provider-->>Core: Model output
    Core->>Server: Tool call if needed
    Server-->>Core: Tool result
    Core->>Session: Sync usage, tokens & append messages
    Core-->>Host: Final response
    Host-->>User: Output
```

## Crate reading order

1. `antikythera-domain` — canonical domain types (start here).
2. `antikythera-ports` — port/adapter trait definitions.
3. `antikythera-config` — configuration schema and loading.
4. `antikythera-log` — unified logging infrastructure.
5. `antikythera-resilience` — retry policies, health tracking, context window management.
6. `antikythera-tooling` — MCP tool server management.
7. `antikythera-session` — session management and chat history.
8. `antikythera-storage` — pluggable session persistence.
9. `antikythera-core` — application layer: protocol, transport, orchestration, streaming (depends on all above).
10. `antikythera-sdk` — SDK/integration surface (FFI boundaries, WASM bindings).

Example applications:
- `examples/antikythera-web` — Web frontend (Vue.js/TypeScript).

Supporting crates:
- `antikythera-wasm-bindgen` — wasm-bindgen bindings for browser targets.

> **Note:** `antikythera-core/src/domain/` and `antikythera-core/src/application/ports/` are now thin re-exports from the `antikythera-domain` and `antikythera-ports` crates respectively. The canonical definitions live in those crates.
