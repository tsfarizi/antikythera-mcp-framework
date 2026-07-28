//! # Antikythera ToolRunner
//!
//! MCP tool registration, validation, and in-process execution.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           ToolRunner                     │
//! │                                          │
//! │  Registry (definitions + validation)     │
//! │  Handlers (builtin tool functions)       │
//! │                                          │
//! │  execute(tool, args)                     │
//! │    ├── builtin? → handler(args)          │
//! │    └── external? → Err(HostRequired)     │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## WASM Integration
//!
//! When compiled to WASM, the `wasm` module provides a bridge that intercepts
//! `emit-tool-call` host-imports and executes builtin tools in-process,
//! eliminating the round-trip to the host for registered tools.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use antikythera_toolrunner::{ToolRunner, ToolDefinition, ToolParameterSchema};
//!
//! let mut runner = ToolRunner::new();
//!
//! // Register tool definition
//! runner.register_tool(ToolDefinition {
//!     name: "get_weather".into(),
//!     description: "Get weather for a city".into(),
//!     parameters: vec![ToolParameterSchema {
//!         name: "city".into(),
//!         param_type: "string".into(),
//!         description: "City name".into(),
//!         required: true,
//!     }],
//!     ..Default::default()
//! });
//!
//! // Register handler
//! runner.register_handler("get_weather", |args| {
//!     let city = args.get("city").and_then(|v| v.as_str()).unwrap_or("unknown");
//!     Ok(serde_json::json!({"temp": 25, "city": city}))
//! });
//!
//! // Execute
//! let result = runner.execute("get_weather", serde_json::json!({"city": "Jakarta"}));
//! assert!(result.is_ok());
//! ```

pub mod error;
pub mod handler;
pub mod registry;
pub mod runner;
pub mod types;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use error::ToolRunnerError;
pub use handler::ToolHandlerFn;
pub use registry::ToolRegistry;
pub use runner::ToolRunner;
pub use types::{ToolCall, ToolDefinition, ToolParameterSchema, ToolResult};
