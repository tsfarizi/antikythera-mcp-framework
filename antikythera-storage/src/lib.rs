pub mod backend;
pub mod backup;
pub mod cache;
pub mod config;
pub mod error;

#[cfg(feature = "standalone")]
pub mod api;

#[cfg(feature = "sse")]
pub mod sse;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(any(feature = "mongodb", feature = "postgres"))]
pub mod schema;

use std::sync::Arc;

use backend::StorageBackend;
use cache::{CacheManager, EvictionPolicy};
use config::StorageConfig;
use error::StorageError;

/// High-level storage coordinator that ties together backend, cache, and backup.
pub struct StorageEngine {
    backend: Arc<dyn StorageBackend>,
    cache: CacheManager,
    config: StorageConfig,
}

impl StorageEngine {
    /// Initialize the storage engine from configuration.
    pub async fn new(config: StorageConfig) -> Result<Self, StorageError> {
        let backend: Arc<dyn StorageBackend> = match config.backend.as_str() {
            "filesystem" => Arc::new(
                backend::filesystem::FilesystemBackend::new(&config.data_dir, &config.backup_dir)
                    .map_err(|e| StorageError::Config(e.to_string()))?,
            ),
            #[cfg(feature = "mongodb")]
            "mongodb" => {
                let backend = backend::mongodb::MongoBackend::new(&config.mongodb)
                    .await
                    .map_err(|e| StorageError::Connection(e.to_string()))?;
                Arc::new(backend)
            }
            #[cfg(feature = "postgres")]
            "postgres" => {
                let backend = backend::postgres::PostgresBackend::new(&config.postgres)
                    .await
                    .map_err(|e| StorageError::Connection(e.to_string()))?;
                Arc::new(backend)
            }
            other => {
                return Err(StorageError::Config(format!(
                    "unknown storage backend: {other}"
                )));
            }
        };

        let cache = CacheManager::new(
            config.cache.max_sessions,
            config.cache.ttl_seconds,
            EvictionPolicy::from(config.cache.eviction_policy.as_str()),
        );

        Ok(Self {
            backend,
            cache,
            config,
        })
    }

    /// Load session data, using cache when available.
    pub async fn load(&mut self, session_id: &str) -> Result<Option<Vec<u8>>, StorageError> {
        // Try cache first.
        if self.config.cache.enabled {
            if let Some(data) = self.cache.get(session_id) {
                return Ok(Some(data));
            }
        }

        // Load from backend.
        let data = self.backend.load(session_id).await?;

        // Populate cache.
        if self.config.cache.enabled {
            if let Some(ref d) = data {
                self.cache
                    .insert(session_id.to_string(), d.clone());
            }
        }

        Ok(data)
    }

    /// Save session data, updating cache and triggering backup.
    pub async fn save(&mut self, session_id: &str, data: Vec<u8>) -> Result<(), StorageError> {
        // Save to backend.
        self.backend.save(session_id, &data).await?;

        // Update cache.
        if self.config.cache.enabled {
            self.cache.insert(session_id.to_string(), data.clone());
        }

        // Trigger backup if enabled.
        if self.config.backup.enabled {
            self.backend.backup(session_id, &data).await?;
        }

        Ok(())
    }

    /// Delete a session from backend and cache.
    pub async fn delete(&mut self, session_id: &str) -> Result<(), StorageError> {
        self.backend.delete(session_id).await?;
        if self.config.cache.enabled {
            self.cache.remove(session_id);
        }
        Ok(())
    }

    /// List all sessions from the backend.
    pub async fn list(&self) -> Result<Vec<String>, StorageError> {
        self.backend.list().await
    }

    /// Check if a session exists.
    pub async fn exists(&self, session_id: &str) -> Result<bool, StorageError> {
        self.backend.exists(session_id).await
    }

    /// Run cache sweep for expired entries.
    pub fn sweep_cache(&mut self) -> Vec<String> {
        if self.config.cache.enabled {
            self.cache.sweep_expired()
        } else {
            Vec::new()
        }
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.cache.len(),
            max_entries: self.config.cache.max_sessions,
            dirty_entries: self.cache.dirty_sessions().len(),
        }
    }

    /// Get a reference to the underlying backend.
    pub fn backend(&self) -> &dyn StorageBackend {
        self.backend.as_ref()
    }

    /// Get the storage configuration.
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }
}

/// Cache statistics snapshot.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub max_entries: usize,
    pub dirty_entries: usize,
}
