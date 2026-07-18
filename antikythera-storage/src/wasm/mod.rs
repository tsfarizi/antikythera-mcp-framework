pub mod ffi;

use crate::config::StorageConfig;
use crate::error::StorageError;
use crate::StorageEngine;

/// WASM-compatible storage wrapper.
pub struct WasmStorage {
    engine: StorageEngine,
}

impl WasmStorage {
    /// Initialize WASM storage from configuration.
    pub async fn new(config: StorageConfig) -> Result<Self, StorageError> {
        let engine = StorageEngine::new(config).await?;
        Ok(Self { engine })
    }

    /// Load session data.
    pub async fn load(&mut self, session_id: &str) -> Result<Option<Vec<u8>>, StorageError> {
        self.engine.load(session_id).await
    }

    /// Save session data.
    pub async fn save(&mut self, session_id: &str, data: Vec<u8>) -> Result<(), StorageError> {
        self.engine.save(session_id, data).await
    }

    /// Delete a session.
    pub async fn delete(&mut self, session_id: &str) -> Result<(), StorageError> {
        self.engine.delete(session_id).await
    }

    /// List all sessions.
    pub async fn list(&self) -> Result<Vec<String>, StorageError> {
        self.engine.list().await
    }
}
