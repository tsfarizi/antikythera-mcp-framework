//! Storage error types.

use std::path::PathBuf;

/// Errors that can occur during storage operations.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Path error: {path}")]
    Path {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Backup error: {0}")]
    Backup(String),

    #[error("Schema error: {0}")]
    Schema(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

impl From<StorageError> for String {
    fn from(e: StorageError) -> Self {
        e.to_string()
    }
}
