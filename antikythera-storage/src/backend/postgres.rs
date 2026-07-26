//! PostgreSQL storage backend using `sqlx`.
//!
//! Sessions are stored in a `sessions` table with JSONB data column.
//! Backups use the local filesystem as intermediate storage before
//! syncing to the database.

use std::path::PathBuf;

use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::types::Json;

use antikythera_domain::session::Session;

use crate::config::PostgresConfig;
use crate::error::StorageError;

use super::StorageBackend;

/// PostgreSQL storage backend using `sqlx`.
///
/// Sessions are stored in a `sessions` table with JSONB data.
/// Backups use the local filesystem as intermediate storage.
pub struct PostgresBackend {
    pool: PgPool,
    backup_dir: PathBuf,
}

impl PostgresBackend {
    /// Create a new PostgreSQL backend.
    ///
    /// Connects to the database using the provided config and optionally
    /// creates the schema if `auto_create_schema` is enabled.
    pub async fn new(config: &PostgresConfig) -> Result<Self, StorageError> {
        let connection_string = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.user, config.password, config.host, config.port, config.database,
        );

        let pool = PgPool::connect(&connection_string)
            .await
            .map_err(|e| StorageError::Connection(e.to_string()))?;

        if config.auto_create_schema {
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS sessions (
                    id VARCHAR(36) PRIMARY KEY,
                    data JSONB NOT NULL,
                    created_at TIMESTAMPTZ DEFAULT NOW(),
                    updated_at TIMESTAMPTZ DEFAULT NOW()
                );
                "#,
            )
            .execute(&pool)
            .await
            .map_err(|e| StorageError::Schema(e.to_string()))?;
        }

        let backup_dir = std::env::current_dir()
            .map_err(StorageError::Io)?
            .join("data")
            .join("backups");

        Ok(Self { pool, backup_dir })
    }
}

#[async_trait]
impl StorageBackend for PostgresBackend {
    async fn save(&self, session_id: &str, data: &[u8]) -> Result<(), StorageError> {
        let session: Session = serde_json::from_slice(data).map_err(StorageError::Serialization)?;

        sqlx::query(
            r#"
            INSERT INTO sessions (id, data, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data, updated_at = NOW();
            "#,
        )
        .bind(session_id)
        .bind(Json(&session))
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let result: Option<Json<Session>> =
            sqlx::query_scalar("SELECT data FROM sessions WHERE id = $1")
                .bind(session_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| StorageError::Backend(e.to_string()))?;

        match result {
            Some(Json(session)) => {
                let bytes = serde_json::to_vec(&session).map_err(StorageError::Serialization)?;
                Ok(Some(bytes))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &str) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(session_id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        Ok(())
    }

    async fn list(&self) -> Result<Vec<String>, StorageError> {
        let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM sessions")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        Ok(ids)
    }

    async fn exists(&self, session_id: &str) -> Result<bool, StorageError> {
        let result: Option<i32> = sqlx::query_scalar("SELECT 1 FROM sessions WHERE id = $1")
            .bind(session_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        Ok(result.is_some())
    }

    async fn backup(&self, session_id: &str, data: &[u8]) -> Result<(), StorageError> {
        let backup_dir = self.backup_dir.clone();
        let session_id = session_id.to_string();
        let data = data.to_vec();
        tokio::task::spawn_blocking(move || {
            std::fs::create_dir_all(&backup_dir)?;
            let path = backup_dir.join(format!("{session_id}.backup.json"));
            std::fs::write(&path, data)
        })
        .await
        .map_err(|e| StorageError::Backend(e.to_string()))?
        .map_err(StorageError::Io)
    }

    async fn sync_backup(&self, session_id: &str) -> Result<(), StorageError> {
        let path = self.backup_dir.join(format!("{session_id}.backup.json"));
        let path_clone = path.clone();
        let bytes = tokio::task::spawn_blocking(move || std::fs::read(&path_clone))
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?
            .map_err(StorageError::Io)?;

        self.save(session_id, &bytes).await?;

        Ok(())
    }

    async fn verify_sync(&self, session_id: &str) -> Result<bool, StorageError> {
        self.exists(session_id).await
    }

    async fn delete_backup(&self, session_id: &str) -> Result<(), StorageError> {
        let path = self.backup_dir.join(format!("{session_id}.backup.json"));
        let path_clone = path.clone();
        tokio::task::spawn_blocking(move || match std::fs::remove_file(&path_clone) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Path { path, source: e }),
        })
        .await
        .map_err(|e| StorageError::Backend(e.to_string()))?
    }
}
