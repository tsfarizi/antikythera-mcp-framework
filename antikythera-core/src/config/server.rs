//! # MCP Server Configuration
//!
//! This module defines configuration for connecting to MCP (Model Context Protocol) servers.
//! MCP servers provide tools that the AI agent can use to perform actions.
//!
//! ## Example - STDIO Server
//!
//! ```toml
//! [[servers]]
//! name = "time"
//! command = "python"
//! args = ["-m", "mcp_server_time"]
//! ```
//!
//! ## Example - HTTP Server
//!
//! ```toml
//! [[servers]]
//! name = "remote-api"
//! url = "https://mcp-server.example.com"
//! ```

pub use super::schema::{ServerConfig, TransportType};

use serde::Deserialize;
use shellexpand;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct RawServer {
    pub name: String,
    /// Command for STDIO transport (optional if url is provided)
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub workdir: Option<String>,
    /// URL for HTTP transport (optional if command is provided)
    pub url: Option<String>,
    /// HTTP headers for authentication
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub default_timezone: Option<String>,
    #[serde(default)]
    pub default_city: Option<String>,
}

impl From<RawServer> for ServerConfig {
    fn from(raw: RawServer) -> Self {
        let expand = |s: &str| -> String {
            shellexpand::full(s)
                .map(|cow| cow.into_owned())
                .unwrap_or_else(|_| s.to_string())
        };

        // Determine transport type based on provided fields
        let (transport, command, url) = if let Some(url_str) = raw.url {
            // HTTP transport
            (TransportType::Http, None, Some(url_str))
        } else if let Some(cmd_str) = raw.command {
            // STDIO transport
            let command_expanded = PathBuf::from(expand(&cmd_str));
            (TransportType::Stdio, Some(command_expanded), None)
        } else {
            // Default to Builtin (in-process handler)
            (TransportType::Builtin, None, None)
        };

        let workdir = raw.workdir.map(|d| PathBuf::from(expand(&d)));
        let args = raw.args.into_iter().map(|arg| expand(&arg)).collect();

        Self {
            name: raw.name,
            transport,
            command,
            args,
            env: raw.env,
            workdir,
            url,
            headers: raw.headers,
            default_timezone: raw.default_timezone,
            default_city: raw.default_city,
        }
    }
}
