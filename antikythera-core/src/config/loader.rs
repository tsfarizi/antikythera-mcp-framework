//! Configuration loader - Postcard-only
//!
//! All configuration is stored as a single Postcard binary file (`app.pc`).

use super::app::AppConfig;
use super::error::ConfigError;
use super::postcard_config;
use crate::logging::ConfigLogger;
use dotenvy::from_filename;
use std::path::Path;
use std::sync::Once;

static ENV_LOADER: Once = Once::new();

/// Load and validate configuration from Postcard binary
pub fn load_config(path: Option<&Path>) -> Result<AppConfig, ConfigError> {
    ENV_LOADER.call_once(|| {
        let _ = from_filename(super::ENV_PATH);
    });

    let config_path = path.unwrap_or_else(|| Path::new(postcard_config::CONFIG_PATH));

    if !config_path.exists() {
        return Err(ConfigError::NotFound {
            path: config_path.to_path_buf(),
        });
    }

    let data = std::fs::read(config_path).map_err(|e| ConfigError::Io {
        path: config_path.to_path_buf(),
        source: e,
    })?;

    let config = match postcard_config::config_from_postcard(&data) {
        Ok(c) => c,
        Err(e) => {
            // The binary file is from an older schema version (Postcard is
            // positional — adding fields invalidates existing blobs).
            // Back up the stale file and write a fresh default so the
            // application can start without manual intervention.
            let logger = ConfigLogger::new("config");
            logger.warn(format!(
                "Config schema changed; existing file is unreadable ({}). \
                 Backing up to {}.bak and writing fresh defaults.",
                e,
                config_path.display()
            ));

            let backup_path = config_path.with_extension("pc.bak");
            let _ = std::fs::copy(config_path, &backup_path);

            let fresh = AppConfig::default();
            if let Ok(fresh_data) = postcard_config::config_to_postcard(&fresh) {
                let _ = std::fs::write(config_path, fresh_data);
            }
            fresh
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

/// Save configuration to Postcard binary
pub fn save_config(config: &AppConfig, path: Option<&Path>) -> Result<(), ConfigError> {
    let config_path = path.unwrap_or_else(|| Path::new(postcard_config::CONFIG_PATH));

    let data = postcard_config::config_to_postcard(config)
        .map_err(|e| ConfigError::CacheError(format!("Postcard serialize error: {}", e)))?;

    if let Some(parent) = config_path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).map_err(|e| ConfigError::Io {
            path: config_path.to_path_buf(),
            source: e,
        })?;
    }

    std::fs::write(config_path, &data).map_err(|e| ConfigError::Io {
        path: config_path.to_path_buf(),
        source: e,
    })?;

    // Log successful save
    let logger = ConfigLogger::new("config");
    logger.info(format!("Config saved to: {}", config_path.display()));
    logger.debug(format!("  Size: {} bytes", data.len()));

    Ok(())
}


