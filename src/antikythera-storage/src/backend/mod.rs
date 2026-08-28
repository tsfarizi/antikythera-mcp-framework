//! Storage backend trait and implementations.
//!
//! All backends implement the [`StorageBackend`] trait for uniform access.

use async_trait::async_trait;

use crate::error::StorageError;

/// Filesystem-based storage backend.
pub mod filesystem;

#[cfg(feature = "mongodb")]
/// MongoDB storage backend.
pub mod mongodb;

#[cfg(feature = "postgres")]
/// PostgreSQL storage backend.
pub mod postgres;

/// Core storage backend trait.
///
/// All storage backends (filesystem, MongoDB, PostgreSQL) implement this trait
/// to provide a uniform interface for session persistence.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Save session data as JSON bytes.
    async fn save(&self, session_id: &str, data: &[u8]) -> Result<(), StorageError>;

    /// Load session data as JSON bytes.
    async fn load(&self, session_id: &str) -> Result<Option<Vec<u8>>, StorageError>;

    /// Delete a session.
    async fn delete(&self, session_id: &str) -> Result<(), StorageError>;

    /// List all session IDs.
    async fn list(&self) -> Result<Vec<String>, StorageError>;

    /// Check if a session exists.
    async fn exists(&self, session_id: &str) -> Result<bool, StorageError>;

    /// Backup data to intermediate storage (filesystem for SQL backends).
    async fn backup(&self, session_id: &str, data: &[u8]) -> Result<(), StorageError>;

    /// Sync backup from intermediate storage to primary storage.
    async fn sync_backup(&self, session_id: &str) -> Result<(), StorageError>;

    /// Verify that backup was synced successfully to primary storage.
    async fn verify_sync(&self, session_id: &str) -> Result<bool, StorageError>;

    /// Delete backup file after successful sync.
    async fn delete_backup(&self, session_id: &str) -> Result<(), StorageError>;
}
