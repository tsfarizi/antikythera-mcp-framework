//! # Antikythera Core
//!
//! Core MCP protocol implementation, transport layers, and agent runtime.

pub mod application;
pub mod config;
pub mod constants;
pub mod domain;
pub mod infrastructure;

/// Security module for input validation, rate limiting, and secrets management
pub mod security;

/// Unified logging system for all core operations
pub mod logging;

// Re-export commonly used types
pub use application::agent::{Agent, AgentOptions, AgentOutcome, ToolDescriptor};
pub use application::agent::events::DomainEvent;

// Re-export resilience module at crate root
pub use application::client::{ChatRequest, ChatResult, ClientConfig, McpClient, PreparedChatTurn};
pub use config::AppConfig;

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
