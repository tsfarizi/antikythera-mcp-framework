//! Application configuration — re-exports schema types and adds format-specific methods.

pub use super::schema::{
    AgentConfig, AppConfig, DocServerConfig, ModelConfig, ModelInfo, ProviderConfig, PromptsConfig,
    RestServerConfig,
};

use super::error::ConfigError;
use std::path::Path;

impl AppConfig {
    /// Load configuration from a TOML file (or the default path).
    pub fn load(path: Option<&Path>) -> Result<Self, ConfigError> {
        super::loader::load_config(path)
    }

    /// Convert configuration to TOML string.
    pub fn to_raw_toml(&self) -> String {
        super::serializer::to_raw_toml_string(self)
    }
}
