//! Postcard binary serialization helpers for [`super::AppConfig`].
//!
//! The canonical struct definition lives in [`super::app`].  This module
//! provides the thin serialize/deserialize/load/save functions that operate
//! on the Postcard binary blob (`app.pc`).

pub use super::app::AppConfig;
use std::path::Path;

/// Configuration file path (project root)
pub const CONFIG_PATH: &str = "app.pc";

/// Serialize configuration to Postcard binary.
pub fn config_to_postcard(config: &AppConfig) -> Result<Vec<u8>, String> {
    postcard::to_allocvec(config).map_err(|e| format!("Failed to serialize config: {}", e))
}

/// Deserialize configuration from Postcard binary.
pub fn config_from_postcard(data: &[u8]) -> Result<AppConfig, String> {
    postcard::from_bytes(data).map_err(|e| format!("Failed to deserialize config: {}", e))
}

/// Load configuration from file.
pub fn load_config(path: Option<&Path>) -> Result<AppConfig, String> {
    let config_path = path.unwrap_or(Path::new(CONFIG_PATH));

    if !config_path.exists() {
        return Err(format!("Config file not found: {}", config_path.display()));
    }

    let data =
        std::fs::read(config_path).map_err(|e| format!("Failed to read config file: {}", e))?;

    config_from_postcard(&data)
}

/// Save configuration to file.
pub fn save_config(config: &AppConfig, path: Option<&Path>) -> Result<(), String> {
    let config_path = path.unwrap_or(Path::new(CONFIG_PATH));

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let data = config_to_postcard(config)?;

    std::fs::write(config_path, &data)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

// ── Backwards-compatible type aliases ────────────────────────────────────────

/// Backwards-compatible alias.  Prefer [`AppConfig`] directly.
pub type PostcardAppConfig = AppConfig;

/// Backwards-compatible alias for the server config sub-struct.
pub type ServerConfig = super::app::RestServerConfig;
