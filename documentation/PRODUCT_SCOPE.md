# Product Scope

This document defines what the Antikythera MCP Framework is, what deployment targets it supports, and what surfaces its public API exposes.

## What it is

Antikythera is a **Rust-based MCP client framework** designed to:

- prepare and process LLM message flows while leaving the actual model API call to the embedding host
- connect to MCP tool servers over STDIO and HTTP transports
- run agent and tool-calling flows with structured step management
- expose agent logic as a portable **server-side WASM component** (wasm32-wasip1)

No browser WASM, no C FFI, and no embedded HTTP server are provided by the framework. A host that embeds the WASM component is responsible for its own transport layer (REST, gRPC, WebSocket, or custom).

## Deployment targets

| Target | Build command | Output |
|:-------|:-------------|:-------|
| **Framework crates** | `cargo build --workspace` | Library crates |
| **Server-side WASM component** | `cargo component build -p antikythera-sdk --release --target wasm32-wasip1` | `.wasm` component |

No browser WASM, no C FFI, and no embedded HTTP server are provided by the framework. A host that embeds the WASM component is responsible for its own transport layer (REST, gRPC, WebSocket, or custom).

## Public SDK surface

The `antikythera-sdk` crate provides the stable integration surface:

| Area | Key types |
|:-----|:---------|
| Client and config | `AppConfig`, `McpClient`, `ClientConfig`, `ChatRequest`, `PreparedChatTurn` |
| Agent infrastructure | `Agent`, `AgentOptions`, `AgentOutcome`, `ToolDescriptor` |
| Host model delegation | `DynamicModelProvider`, `ModelProvider`, `HostModelClient`, `HostModelTransport` |
| Multi-agent | `MultiAgentOrchestrator`, `AgentProfile`, `AgentTask` |
| Routing strategies | `DirectRouter`, `RoundRobinRouter`, `FirstAvailableRouter`, `RoleRouter` |
| Logging | `AgentLogger`, `ChatLogger`, `ConfigLogger`, `DiscoveryLogger`, `OrchestratorLogger`, `ProviderLogger`, `ResilienceLogger`, `SecurityLogger`, `SessionLogger`, `StdioLogger`, `StreamingLogger`, `TransportLogger`, `WasmLogger` |
| Session | Session history types, import/export |

## Architecture philosophy

The framework is designed around one principle: **the host owns the interface layer**.

```mermaid
flowchart LR
    HOST[Host application] --> WASM[WASM component]
    HOST --> LLM[LLM provider]
    HOST --> TOOLS[MCP tool servers]
    HOST --> TRANSPORT[Transport: REST / gRPC / custom]
    WASM --> LOGIC[Agent logic and reasoning loop]
```

The WASM component handles agent reasoning, session continuity, history shaping, and response parsing. The host handles every external integration: LLM calls, tool execution, persistence, and protocol exposure. This keeps the component portable across runtimes and avoids embedding infrastructure concerns inside the framework.

## Feature flags

### `antikythera-sdk`

| Flag | Purpose | Status |
|:-----|:--------|:-------|
| `sdk-core` | Re-exports core types (Agent, McpClient, AppConfig) | Stable |
| `single-agent` | Single-agent support | Stable |
| `multi-agent` | Multi-agent orchestration runtime | Stable |
| `component` | Server-side WASM component bindings | Active development |
| `wasm` | Browser WASM support (wasm32-unknown-unknown) | Active development |
| `wasm-sandbox` | Wasmtime host for running WASM agents | Active development |
| `subscriber` | Real-time log streaming via tokio channels | Stable |
| `full` | Enables all features | Stable |

### `antikythera-core`

| Flag | Purpose | Status |
|:-----|:--------|:-------|
| `native-transport` | OS process and stdio transport support | Stable |
| `wizard` | Interactive setup and wizard-related dependencies | Stable |
| `multi-agent` | Multi-agent orchestration support | Stable |
| `full` | Enables the full capability set | Stable |

### `antikythera-log`

| Flag | Purpose | Status |
|:-----|:--------|:-------|
| `wasm` | Browser-safe time via js-sys (wasm32-unknown-unknown) | Stable |
| `subscriber` | Real-time log streaming via tokio + crossbeam-channel | Stable |
| `lint` | Compile-time lint blocking println!, eprintln!, dbg!, tracing | Stable |

### `antikythera-storage`

| Flag | Purpose | Status |
|:-----|:--------|:-------|
| `filesystem` | JSON file storage backend (default) | Stable |
| `mongodb` | MongoDB backend | Stable |
| `postgres` | PostgreSQL backend | Stable |
| `standalone` | REST API server mode | Stable |
| `sse` | SSE backup service | Stable |
| `wasm` | WASM component integration | Stable |

## Related documents

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — crate relationships and request flow
- [`BUILD.md`](BUILD.md) — build commands for each target
- [`COMPONENT.md`](COMPONENT.md) — WASM component model details
- [`WASM_AGENT.md`](WASM_AGENT.md) — agent logic inside the component
