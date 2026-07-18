//! Filesystem-based storage backend.
//!
//! Sessions are stored as JSON files. This is the default backend
//! and does not require any external database.

use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;

use crate::error::StorageError;

use super::StorageBackend;

/// Filesystem-based storage backend.
///
/// Sessions are stored as JSON files at `{data_dir}/{session_id}.json`.
/// Backups are stored at `{backup_dir}/{session_id}.backup.json`.
pub struct FilesystemBackend {
    data_dir: PathBuf,
    backup_dir: PathBuf,
}

impl FilesystemBackend {
    /// Create a new filesystem backend, creating directories if they don't exist.
    pub fn new(data_dir: &PathBuf, backup_dir: &PathBuf) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(data_dir)?;
        std::fs::create_dir_all(backup_dir)?;
        Ok(Self {
            data_dir: data_dir.clone(),
            backup_dir: backup_dir.clone(),
        })
    }

    fn session_path(&self, session_id: &str) -> PathBuf {
        self.data_dir.join(format!("{session_id}.json"))
    }

    fn backup_path(&self, session_id: &str) -> PathBuf {
        self.backup_dir.join(format!("{session_id}.backup.json"))
    }
}

#[async_trait]
impl StorageBackend for FilesystemBackend {
    async fn save(&self, session_id: &str, data: &[u8]) -> Result<(), StorageError> {
        let path = self.session_path(session_id);
        fs::write(&path, data).await.map_err(|e| StorageError::Path {
            path,
            source: e,
        })
    }

    async fn load(&self, session_id: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let path = self.session_path(session_id);
        match fs::read(&path).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::Path {
                path,
                source: e,
            }),
        }
    }

    async fn delete(&self, session_id: &str) -> Result<(), StorageError> {
        let path = self.session_path(session_id);
        match fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Path {
                path,
                source: e,
            }),
        }
    }

    async fn list(&self) -> Result<Vec<String>, StorageError> {
        let mut entries = fs::read_dir(&self.data_dir)
            .await
            .map_err(StorageError::Io)?;
        let mut ids = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(StorageError::Io)?
        {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(id) = name.strip_suffix(".json") {
                ids.push(id.to_string());
            }
        }
        Ok(ids)
    }

    async fn exists(&self, session_id: &str) -> Result<bool, StorageError> {
        let path = self.session_path(session_id);
        match fs::metadata(&path).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(StorageError::Path {
                path,
                source: e,
            }),
        }
    }

    async fn backup(&self, session_id: &str, data: &[u8]) -> Result<(), StorageError> {
        let path = self.backup_path(session_id);
        fs::write(&path, data).await.map_err(|e| StorageError::Path {
            path,
            source: e,
        })
    }

    async fn sync_backup(&self, session_id: &str) -> Result<(), StorageError> {
        let backup = self.backup_path(session_id);
        let target = self.session_path(session_id);
        fs::copy(&backup, &target)
            .await
            .map_err(|e| StorageError::Backup(format!(
                "failed to sync backup for {session_id}: {e}"
            )))?;
        Ok(())
    }

    async fn verify_sync(&self, session_id: &str) -> Result<bool, StorageError> {
        let path = self.session_path(session_id);
        match fs::metadata(&path).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(StorageError::Path {
                path,
                source: e,
            }),
        }
    }

    async fn delete_backup(&self, session_id: &str) -> Result<(), StorageError> {
        let path = self.backup_path(session_id);
        match fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Path {
                path,
                source: e,
            }),
        }
    }
}
