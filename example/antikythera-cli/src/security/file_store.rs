//! CLI-specific file-based secret storage backend.
//!
//! Provides persistent secret storage backed by a JSON file on disk,
//! with versioning support compatible with the security crate's API.

use antikythera_core::security::config::SecretMetadata;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// A single versioned secret entry persisted to disk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileStoredSecret {
    pub value: String,
    pub metadata: SecretMetadata,
}

/// Errors from file-backed secret operations.
#[derive(Debug, Error)]
pub enum FileStoreError {
    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// File-backed secret storage with versioning.
pub struct FileSecretStore {
    path: PathBuf,
    secrets: HashMap<String, Vec<FileStoredSecret>>,
}

impl FileSecretStore {
    /// Open or create a file-backed secret store at `path`.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, FileStoreError> {
        let path = path.into();
        let secrets = if path.exists() {
            Self::load_from_disk(&path)?
        } else {
            HashMap::new()
        };
        Ok(Self { path, secrets })
    }

    /// Store a new version of a secret.
    pub fn store_secret(&mut self, id: &str, value: &str) -> Result<(), FileStoreError> {
        let entry = self.secrets.entry(id.to_string()).or_default();
        let version = entry.len() as u32 + 1;
        let metadata = SecretMetadata::new(id.to_string(), version);
        entry.push(FileStoredSecret {
            value: value.to_string(),
            metadata,
        });
        self.persist()
    }

    /// Retrieve the latest active version of a secret.
    pub fn get_secret(&self, id: &str) -> Result<String, FileStoreError> {
        let entry = self.secrets.get(id).ok_or_else(|| {
            FileStoreError::SecretNotFound(id.to_string())
        })?;
        let latest = entry
            .iter()
            .filter(|s| s.metadata.active)
            .max_by_key(|s| s.metadata.version)
            .ok_or_else(|| FileStoreError::SecretNotFound(id.to_string()))?;
        Ok(latest.value.clone())
    }

    /// Delete all versions of a secret.
    pub fn delete_secret(&mut self, id: &str) -> Result<(), FileStoreError> {
        self.secrets.remove(id).ok_or_else(|| {
            FileStoreError::SecretNotFound(id.to_string())
        })?;
        self.persist()
    }

    /// List all secret IDs.
    pub fn list_secrets(&self) -> Vec<String> {
        self.secrets.keys().cloned().collect()
    }

    /// Persist the current state to disk.
    fn persist(&self) -> Result<(), FileStoreError> {
        let json = serde_json::to_string_pretty(&self.secrets)
            .map_err(|e| FileStoreError::Serialization(e.to_string()))?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| FileStoreError::Io(e.to_string()))?;
        }
        fs::write(&self.path, json)
            .map_err(|e| FileStoreError::Io(e.to_string()))
    }

    fn load_from_disk(path: &std::path::Path) -> Result<HashMap<String, Vec<FileStoredSecret>>, FileStoreError> {
        let content = fs::read_to_string(path)
            .map_err(|e| FileStoreError::Io(e.to_string()))?;
        serde_json::from_str(&content)
            .map_err(|e| FileStoreError::Serialization(e.to_string()))
    }
}
