//! Discovery Types
//!
//! This module defines the core types used for MCP server discovery.

use std::path::PathBuf;
use thiserror::Error;

/// Information about a discovered MCP server binary.
#[derive(Debug, Clone)]
pub struct DiscoveredServer {
    /// Server name derived from binary filename (without extension)
    pub name: String,
    /// Full path to the server binary
    pub binary_path: PathBuf,
    /// List of tools: (tool_name, description)
    pub tools: Vec<(String, String)>,
    /// Status of loading this server
    pub load_status: LoadStatus,
}

impl DiscoveredServer {
    pub fn new(name: impl Into<String>, binary_path: PathBuf) -> Self {
        Self {
            name: name.into(),
            binary_path,
            tools: Vec::new(),
            load_status: LoadStatus::Pending,
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self.load_status, LoadStatus::Success)
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

/// Status of loading a server and fetching its tools.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadStatus {
    Pending,
    Success,
    Failed(String),
    NoTools,
}

impl LoadStatus {
    pub fn is_success(&self) -> bool {
        matches!(self, LoadStatus::Success)
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            LoadStatus::Failed(msg) => Some(msg),
            _ => None,
        }
    }
}

/// Summary of a discovery operation.
#[derive(Debug, Clone, Default)]
pub struct DiscoverySummary {
    pub total_found: usize,
    pub loaded: usize,
    pub failed: usize,
    pub no_tools: usize,
    pub total_tools: usize,
}

impl DiscoverySummary {
    pub fn from_servers(servers: &[DiscoveredServer]) -> Self {
        let mut summary = Self {
            total_found: servers.len(),
            ..Self::default()
        };

        for server in servers {
            match &server.load_status {
                LoadStatus::Success => {
                    summary.loaded += 1;
                    summary.total_tools += server.tools.len();
                }
                LoadStatus::Failed(_) => summary.failed += 1,
                LoadStatus::NoTools => summary.no_tools += 1,
                LoadStatus::Pending => {}
            }
        }

        summary
    }
}

/// Errors that can occur during server discovery.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Servers folder not found: {path}")]
    FolderNotFound { path: PathBuf },

    #[error("Failed to read servers folder: {source}")]
    ReadError {
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to load server '{server}': {message}")]
    LoadError { server: String, message: String },

    #[error("No executable files found in servers folder")]
    NoExecutables,
}
