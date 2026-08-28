//! # antikythera-server-runtime
//!
//! Server host runtime for the Antikythera WASM composite: loads
//! `dist/antikythera-sdk.wasm` in wasmtime, wires the `runtime-hooks` and
//! `host-imports` imports behind default-deny permission gates, runs the K1
//! tool loop, routes tool calls to `local`/`remote`/`mcp` destinations, and
//! exposes the HTTP + SSE wire protocol (`documentation/WIRE_PROTOCOL.md`).

pub mod config;
pub mod control;
pub mod core;
pub mod host;
pub mod http;
pub mod llm;
pub mod loop_owner;
pub mod mcp;
pub mod registry;
pub mod routing;
pub mod wire;
pub mod wit;

pub use config::{GatePolicy, HookName, LlmProviderSpec, ServerRuntimeConfig};
pub use control::{ControlChannel, PendingKind};
pub use core::{CoreSession, RuntimeServer};
pub use loop_owner::{LoopOutcome, ToolLoopConfig};
pub use registry::{Destination, ToolOwner, UnionRegistry};
pub use wire::{
    EventEnvelope, LlmRequest, LlmResponse, PostbackBody, ToolCallEvent, ToolDefinition,
    ToolExecutionResult,
};
