use std::path::Path;
use std::sync::Arc;

use antikythera_storage::config::StorageConfig;
use antikythera_storage::StorageEngine;
use tokio::sync::Mutex;

use crate::error::{CliError, CliResult};

/// Initialize the storage engine from app.toml configuration.
///
/// Reads the `[storage]` section from the TOML config and constructs
/// the appropriate backend (filesystem, MongoDB, or PostgreSQL).
pub async fn init_storage(config_path: Option<&Path>) -> CliResult<Arc<Mutex<StorageEngine>>> {
    let storage_config = load_storage_config(config_path)?;
    let engine = StorageEngine::new(storage_config)
        .await
        .map_err(|e| CliError::Config(format!("storage initialization failed: {e}")))?;
    Ok(Arc::new(Mutex::new(engine)))
}

/// Load storage configuration from the TOML file.
///
/// Falls back to defaults if the `[storage]` section is absent.
pub fn load_storage_config(config_path: Option<&Path>) -> CliResult<StorageConfig> {
    let path = config_path.unwrap_or(Path::new("app.toml"));

    if !path.exists() {
        return Ok(StorageConfig::default());
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| CliError::Config(format!("failed to read config: {e}")))?;

    // Parse the full TOML and extract the [storage] section.
    let full_config: toml::Value = toml::from_str(&content)
        .map_err(|e| CliError::Config(format!("failed to parse config: {e}")))?;

    let storage_table = full_config
        .get("storage")
        .cloned()
        .unwrap_or(toml::Value::Table(toml::map::Map::new()));

    let storage_config: StorageConfig = storage_table
        .try_into()
        .map_err(|e| CliError::Config(format!("invalid storage config: {e}")))?;

    Ok(storage_config)
}

/// Print storage initialization status to the CLI output.
pub fn print_storage_status(config: &StorageConfig) {
    use antikythera_log::cli_eprint;

    cli_eprint!(
        "[storage] backend={} mode={} cache={} backup={}",
        config.backend,
        config.mode,
        if config.cache.enabled { "on" } else { "off" },
        if config.backup.enabled { "on" } else { "off" },
    );

    if config.is_filesystem() {
        cli_eprint!("[storage] data_dir={}", config.data_dir.display());
    } else if config.is_mongodb() {
        cli_eprint!("[storage] mongodb uri={}", config.mongodb.uri);
    } else if config.is_postgres() {
        cli_eprint!(
            "[storage] postgres {}:{}/{}",
            config.postgres.host,
            config.postgres.port,
            config.postgres.database
        );
    }
}
