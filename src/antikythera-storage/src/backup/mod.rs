//! Backup coordination between cache and persistent storage.
//!
//! Manages the lifecycle of backup files: creation, synchronization to
//! primary storage, verification, and cleanup.

/// Periodic backup synchronization scheduler.
pub mod scheduler;

/// Backup verification utilities.
pub mod verifier;

use std::sync::Arc;

use crate::backend::StorageBackend;
use crate::config::BackupConfig;
use crate::error::StorageError;

/// Coordinates backup operations for dirty cache entries.
pub struct BackupCoordinator {
    backend: Arc<dyn StorageBackend>,
    config: BackupConfig,
}

impl BackupCoordinator {
    /// Create a new backup coordinator.
    pub fn new(backend: Arc<dyn StorageBackend>, config: BackupConfig) -> Self {
        Self { backend, config }
    }

    /// Backup a single dirty session to intermediate storage.
    pub async fn backup_session(&self, session_id: &str, data: &[u8]) -> Result<(), StorageError> {
        self.backend.backup(session_id, data).await
    }

    /// Sync all pending backups from filesystem to primary storage.
    /// Returns (success_count, failure_count).
    pub async fn sync_pending(
        &self,
        backup_dir: &std::path::Path,
    ) -> Result<(usize, usize), StorageError> {
        let mut success = 0;
        let mut failure = 0;

        let backup_dir = backup_dir.to_path_buf();
        let entries: Vec<std::path::PathBuf> = tokio::task::spawn_blocking(move || {
            let mut result = Vec::new();
            if let Ok(read_dir) = std::fs::read_dir(&backup_dir) {
                for entry in read_dir.flatten() {
                    result.push(entry.path());
                }
            }
            result
        })
        .await
        .map_err(|e| StorageError::Backup(format!("failed to read backup dir: {e}")))?;

        for path in entries {
            if path.extension().and_then(|s| s.to_str()) == Some("json")
                && path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.ends_with(".backup"))
                    .unwrap_or(false)
            {
                let stem = path.file_stem().unwrap().to_str().unwrap();
                let session_id = stem.strip_suffix(".backup").unwrap_or(stem);

                match self.sync_single(session_id, &path).await {
                    Ok(()) => success += 1,
                    Err(e) => {
                        tracing::warn!("backup sync failed for {session_id}: {e}");
                        failure += 1;
                    }
                }
            }
        }

        Ok((success, failure))
    }

    /// Sync a single backup file to primary storage.
    async fn sync_single(
        &self,
        session_id: &str,
        backup_path: &std::path::Path,
    ) -> Result<(), StorageError> {
        let path = backup_path.to_path_buf();
        let data = tokio::task::spawn_blocking(move || std::fs::read(&path))
            .await
            .map_err(|e| StorageError::Backup(format!("failed to read backup: {e}")))?
            .map_err(|e| StorageError::Backup(format!("failed to read backup: {e}")))?;

        self.backend.save(session_id, &data).await?;

        if self.config.verify_before_delete
            && !verifier::verify_sync(self.backend.as_ref(), session_id).await?
        {
            return Err(StorageError::Backup(format!(
                "verification failed after sync for {session_id}"
            )));
        }

        let path = backup_path.to_path_buf();
        tokio::task::spawn_blocking(move || std::fs::remove_file(&path))
            .await
            .map_err(|e| StorageError::Backup(format!("failed to delete backup: {e}")))?
            .map_err(|e| StorageError::Backup(format!("failed to delete backup: {e}")))?;

        Ok(())
    }

    /// Check if backup is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the backup configuration.
    pub fn config(&self) -> &BackupConfig {
        &self.config
    }
}
