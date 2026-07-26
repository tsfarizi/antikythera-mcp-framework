//! # Configuration Module
//!
//! This module handles all configuration loading, parsing, and validation for the MCP client.
//!
//! ## Configuration
//!
//! The client uses a single TOML configuration file:
//!
//! - **`app.toml`** - All settings (providers, model, prompts, agent, server)
//!
//! ## Key Types
//!
//! - [`AppConfig`] - Unified runtime + TOML configuration
//! - [`PromptsConfig`] - Configurable prompts for agent behavior
//! - [`ToolConfig`] - Tool definition synced from MCP servers
//! - [`ServerConfig`] - MCP server connection settings
//! - [`ModelConfig`] - Default provider and model routing
//! - [`ProviderConfig`] - LLM provider definition

pub mod app;
pub mod error;
pub mod loader;
pub mod serializer;
pub mod server;
pub mod tool;

/// TOML serialization helpers
pub mod toml_config;

/// Format-agnostic schema definitions (struct fields, defaults, accessors).
pub mod schema;

pub use app::{
    AgentConfig, AppConfig, DocServerConfig, ModelConfig, ModelInfo, ProviderConfig, PromptsConfig,
    RestServerConfig,
};
pub use error::ConfigError;
pub use server::{ServerConfig, TransportType};
pub use tool::ToolConfig;
pub use toml_config::{CONFIG_PATH, ENV_PATH};
