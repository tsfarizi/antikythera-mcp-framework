//! CLI Infrastructure Layer
//!
//! External services: LLM providers, MCP clients, config loading.
//! Implements the domain ports (interfaces).

pub mod history;
pub mod llm;
pub mod mcp;
pub mod transport;

pub use llm::*;
