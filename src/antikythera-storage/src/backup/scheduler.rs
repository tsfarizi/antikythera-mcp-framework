//! Periodic scheduler for syncing backup files to primary storage.

use std::path::PathBuf;
use std::sync::Arc;

use crate::backend::StorageBackend;
use crate::config::BackupConfig;
use crate::error::StorageError;

use super::BackupCoordinator;

/// Periodic scheduler that syncs backup files to primary storage.
pub struct BackupScheduler {
    coordinator: BackupCoordinator,
    backup_dir: PathBuf,
    interval_seconds: u64,
}

impl BackupScheduler {
    /// Create a new backup scheduler.
    pub fn new(
        backend: Arc<dyn StorageBackend>,
        config: BackupConfig,
        backup_dir: PathBuf,
    ) -> Self {
        let interval_seconds = config.sync_interval_seconds;
        Self {
            coordinator: BackupCoordinator::new(backend, config),
            backup_dir,
            interval_seconds,
        }
    }

    /// Run a single sync cycle.
    pub async fn run_sync(&self) -> Result<(usize, usize), StorageError> {
        self.coordinator.sync_pending(&self.backup_dir).await
    }

    /// Get the sync interval in seconds.
    pub fn interval_seconds(&self) -> u64 {
        self.interval_seconds
    }

    /// Get a reference to the coordinator.
    pub fn coordinator(&self) -> &BackupCoordinator {
        &self.coordinator
    }
}
