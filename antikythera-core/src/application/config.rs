//! Application-level configuration types.
//!
//! These are the data structures that application code depends on.
//! The config module provides serialization/deserialization (outer ring).
//! Application code imports from HERE, not from crate::config.

pub use crate::config::{
    AgentConfig, AppConfig, DocServerConfig, ModelConfig, ModelInfo, PromptsConfig, ProviderConfig,
    RestServerConfig,
};
pub use crate::config::server::{ServerConfig, TransportType};
pub use crate::config::tool::ToolConfig;
pub use crate::constants::CONFIG_PATH;
