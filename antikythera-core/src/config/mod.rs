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
#[cfg(feature = "wizard")]
pub mod wizard;

/// TOML serialization helpers
pub mod toml_config;

/// Backward-compatible alias — delegates to [`toml_config`].
///
/// **Deprecated**: use [`toml_config`] directly.
pub mod postcard_config {
    pub use super::toml_config::*;

    /// Backwards-compatible type alias.
    pub type PostcardAppConfig = super::toml_config::AppConfig;

    /// Serialize configuration to bytes (TOML text encoded as UTF-8).
    pub fn config_to_postcard(
        config: &super::toml_config::AppConfig,
    ) -> Result<Vec<u8>, String> {
        super::toml_config::config_to_toml(config)
            .map(|s| s.into_bytes())
    }

    /// Deserialize configuration from bytes (UTF-8 TOML text).
    pub fn config_from_postcard(data: &[u8]) -> Result<super::toml_config::AppConfig, String> {
        let s = std::str::from_utf8(data)
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;
        super::toml_config::config_from_toml(s)
    }
}

pub use crate::constants::{CONFIG_PATH, ENV_PATH};

pub use app::{
    AgentConfig, AppConfig, DocServerConfig, ModelConfig, ModelInfo, ProviderConfig, PromptsConfig,
    RestServerConfig,
};
pub use error::ConfigError;
pub use server::{ServerConfig, TransportType};
pub use tool::ToolConfig;

// Re-export logging for config operations
pub use crate::logging::ConfigLogger;
