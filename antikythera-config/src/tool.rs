//! # Tool Configuration
//!
//! This module defines tool configuration synced from MCP servers.
//! Tools are capabilities provided by MCP servers that the AI agent can invoke.

pub use super::schema::ToolConfig;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum RawTool {
    Name(String),
    Detailed {
        name: String,
        description: Option<String>,
        #[serde(default)]
        server: Option<String>,
    },
}

impl From<RawTool> for ToolConfig {
    fn from(value: RawTool) -> Self {
        match value {
            RawTool::Name(name) => Self {
                name,
                description: None,
                server: None,
            },
            RawTool::Detailed {
                name,
                description,
                server,
            } => Self {
                name,
                description,
                server,
            },
        }
    }
}
