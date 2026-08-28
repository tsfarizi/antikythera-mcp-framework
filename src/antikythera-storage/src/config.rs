//! Configuration types for the storage layer.
//!
//! All settings are deserialized from the `[storage]` section in `app.toml`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Root storage configuration, deserialized from `[storage]` in `app.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Backend type: "filesystem" | "mongodb" | "postgres"
    #[serde(default = "default_backend")]
    pub backend: String,

    /// Data directory for filesystem backend.
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// Backup directory for SQL backends (intermediate filesystem backup).
    #[serde(default = "default_backup_dir")]
    pub backup_dir: PathBuf,

    /// Deployment mode: "embedded" | "standalone"
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Cache configuration.
    #[serde(default)]
    pub cache: CacheConfig,

    /// Backup configuration.
    #[serde(default)]
    pub backup: BackupConfig,

    /// MongoDB backend configuration.
    #[serde(default)]
    pub mongodb: MongodbConfig,

    /// PostgreSQL backend configuration.
    #[serde(default)]
    pub postgres: PostgresConfig,

    /// SSE backup service configuration.
    #[serde(default)]
    pub sse_backup: SseBackupConfig,
}

fn default_backend() -> String {
    "filesystem".to_string()
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("./data/sessions")
}

fn default_backup_dir() -> PathBuf {
    PathBuf::from("./data/backups")
}

fn default_mode() -> String {
    "embedded".to_string()
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            data_dir: default_data_dir(),
            backup_dir: default_backup_dir(),
            mode: default_mode(),
            cache: CacheConfig::default(),
            backup: BackupConfig::default(),
            mongodb: MongodbConfig::default(),
            postgres: PostgresConfig::default(),
            sse_backup: SseBackupConfig::default(),
        }
    }
}

/// In-memory cache configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable/disable caching.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum number of sessions to hold in RAM.
    #[serde(default = "default_max_sessions")]
    pub max_sessions: usize,

    /// Time-to-live in seconds for cached sessions.
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,

    /// Eviction policy: "lru" | "ttl" | "both"
    #[serde(default = "default_eviction_policy")]
    pub eviction_policy: String,
}

fn default_true() -> bool {
    true
}

fn default_max_sessions() -> usize {
    512
}

fn default_ttl() -> u64 {
    3600
}

fn default_eviction_policy() -> String {
    "both".to_string()
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            max_sessions: default_max_sessions(),
            ttl_seconds: default_ttl(),
            eviction_policy: default_eviction_policy(),
        }
    }
}

/// Backup coordination configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Enable/disable backup.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Backup mode: "realtime" | "interval"
    #[serde(default = "default_backup_mode")]
    pub mode: String,

    /// Sync interval in seconds for interval mode.
    #[serde(default = "default_sync_interval")]
    pub sync_interval_seconds: u64,

    /// Ensure DB success before deleting backup file.
    #[serde(default = "default_true")]
    pub verify_before_delete: bool,
}

fn default_backup_mode() -> String {
    "realtime".to_string()
}

fn default_sync_interval() -> u64 {
    30
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            mode: default_backup_mode(),
            sync_interval_seconds: default_sync_interval(),
            verify_before_delete: default_true(),
        }
    }
}

/// MongoDB backend configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongodbConfig {
    /// MongoDB connection URI.
    #[serde(default = "default_mongodb_uri")]
    pub uri: String,

    /// Database name.
    #[serde(default = "default_mongodb_database")]
    pub database: String,

    /// Collection name for sessions.
    #[serde(default = "default_mongodb_collection")]
    pub collection: String,

    /// Auto-create schema on first connect.
    #[serde(default = "default_true")]
    pub auto_create_schema: bool,
}

fn default_mongodb_uri() -> String {
    "mongodb://localhost:27017".to_string()
}

fn default_mongodb_database() -> String {
    "antikythera".to_string()
}

fn default_mongodb_collection() -> String {
    "sessions".to_string()
}

impl Default for MongodbConfig {
    fn default() -> Self {
        Self {
            uri: default_mongodb_uri(),
            database: default_mongodb_database(),
            collection: default_mongodb_collection(),
            auto_create_schema: default_true(),
        }
    }
}

/// PostgreSQL backend configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    /// Database host.
    #[serde(default = "default_postgres_host")]
    pub host: String,

    /// Database port.
    #[serde(default = "default_postgres_port")]
    pub port: u16,

    /// Database name.
    #[serde(default = "default_postgres_database")]
    pub database: String,

    /// Database user.
    #[serde(default = "default_postgres_user")]
    pub user: String,

    /// Database password.
    #[serde(default)]
    pub password: String,

    /// Auto-create schema on first connect.
    #[serde(default = "default_true")]
    pub auto_create_schema: bool,
}

fn default_postgres_host() -> String {
    "localhost".to_string()
}

fn default_postgres_port() -> u16 {
    5432
}

fn default_postgres_database() -> String {
    "antikythera".to_string()
}

fn default_postgres_user() -> String {
    "postgres".to_string()
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            host: default_postgres_host(),
            port: default_postgres_port(),
            database: default_postgres_database(),
            user: default_postgres_user(),
            password: String::new(),
            auto_create_schema: default_true(),
        }
    }
}

/// SSE backup service configuration for independent backup process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseBackupConfig {
    /// Enable/disable SSE backup service.
    #[serde(default)]
    pub enabled: bool,

    /// Bind address for SSE server.
    #[serde(default = "default_sse_bind")]
    pub bind: String,

    /// Core URL to receive backup events from.
    #[serde(default = "default_core_url")]
    pub core_url: String,
}

fn default_sse_bind() -> String {
    "0.0.0.0:8081".to_string()
}

fn default_core_url() -> String {
    "http://127.0.0.1:8080".to_string()
}

impl Default for SseBackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind: default_sse_bind(),
            core_url: default_core_url(),
        }
    }
}

impl StorageConfig {
    /// Load configuration from a TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Check if backend is filesystem.
    pub fn is_filesystem(&self) -> bool {
        self.backend == "filesystem"
    }

    /// Check if backend is MongoDB.
    pub fn is_mongodb(&self) -> bool {
        self.backend == "mongodb"
    }

    /// Check if backend is PostgreSQL.
    pub fn is_postgres(&self) -> bool {
        self.backend == "postgres"
    }

    /// Check if running in standalone mode.
    pub fn is_standalone(&self) -> bool {
        self.mode == "standalone"
    }

    /// Check if running in embedded mode.
    pub fn is_embedded(&self) -> bool {
        self.mode == "embedded"
    }
}
