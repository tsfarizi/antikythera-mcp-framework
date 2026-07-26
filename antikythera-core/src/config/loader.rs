//! Configuration loader - TOML
//!
//! All configuration is stored as a single TOML file (`app.toml`).

use super::app::AppConfig;
use super::error::ConfigError;
use super::toml_config;
use crate::logging::ConfigLogger;
use dotenvy::from_filename;
use std::path::Path;
use std::sync::Once;

static ENV_LOADER: Once = Once::new();

/// Load and validate configuration from TOML
pub fn load_config(path: Option<&Path>) -> Result<AppConfig, ConfigError> {
    ENV_LOADER.call_once(|| {
        let _ = from_filename(super::ENV_PATH);
    });

    let config_path = path.unwrap_or_else(|| Path::new(toml_config::CONFIG_PATH));

    if !config_path.exists() {
        return Err(ConfigError::NotFound {
            path: config_path.to_path_buf(),
        });
    }

    let data = std::fs::read_to_string(config_path).map_err(|e| ConfigError::Io {
        path: config_path.to_path_buf(),
        source: e,
    })?;

    let config = match toml_config::config_from_toml(&data) {
        Ok(c) => c,
        Err(e) => {
            let backup_path = config_path.with_extension("toml.bak");
            let _ = std::fs::copy(config_path, &backup_path);

            let logger = ConfigLogger::new("config");
            logger.warn(format!(
                "Config schema changed; existing file is unreadable ({}). \
                 Backup saved to {}.",
                e,
                backup_path.display()
            ));

            return Err(ConfigError::SchemaChanged {
                path: config_path.to_path_buf(),
                backup_path,
                reason: e.to_string(),
            });
        }
    };

    // Log successful load
    let logger = ConfigLogger::new("config");
    logger.info(format!("Config loaded from: {}", config_path.display()));
    logger.debug(format!(
        "  Routing: {}/{}",
        config.model.default_provider, config.model.model
    ));

    Ok(config)
}

/// Save configuration to TOML
pub fn save_config(config: &AppConfig, path: Option<&Path>) -> Result<(), ConfigError> {
    let config_path = path.unwrap_or_else(|| Path::new(toml_config::CONFIG_PATH));

    let data = toml_config::config_to_toml(config)
        .map_err(|e| ConfigError::CacheError(format!("TOML serialize error: {}", e)))?;

    if let Some(parent) = config_path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).map_err(|e| ConfigError::Io {
            path: config_path.to_path_buf(),
            source: e,
        })?;
    }

    std::fs::write(config_path, data.as_bytes()).map_err(|e| ConfigError::Io {
        path: config_path.to_path_buf(),
        source: e,
    })?;

    // Log successful save
    let logger = ConfigLogger::new("config");
    logger.info(format!("Config saved to: {}", config_path.display()));
    logger.debug(format!("  Size: {} bytes", data.len()));

    Ok(())
}
