//! CLI Configuration
//!
//! Re-exports the unified `AppConfig` from `antikythera_core::config` and
//! provides thin helper functions for the CLI binary.

use crate::error::{CliError, CliResult};
pub use antikythera_core::config::app::{
    AgentConfig, AppConfig, DocServerConfig, ModelConfig, ModelInfo, ProviderConfig, PromptsConfig,
    RestServerConfig,
};
pub use antikythera_core::config::toml_config::{config_from_toml, config_to_toml, CONFIG_PATH};

use std::path::Path;

fn default_provider_catalog() -> Vec<ProviderConfig> {
    vec![
        ProviderConfig {
            id: "ollama".to_string(),
            provider_type: "ollama".to_string(),
            endpoint: "http://127.0.0.1:11434".to_string(),
            api_key: String::new(),
            models: vec![],
        },
        ProviderConfig {
            id: "gemini".to_string(),
            provider_type: "gemini".to_string(),
            endpoint: "https://generativelanguage.googleapis.com".to_string(),
            api_key: "GEMINI_API_KEY".to_string(),
            models: vec![],
        },
        ProviderConfig {
            id: "openai".to_string(),
            provider_type: "openai".to_string(),
            endpoint: "https://api.openai.com".to_string(),
            api_key: "OPENAI_API_KEY".to_string(),
            models: vec![],
        },
    ]
}

pub fn recommended_default_config() -> AppConfig {
    AppConfig {
        providers: default_provider_catalog(),
        model: ModelConfig {
            default_provider: "ollama".to_string(),
            model: "llama3.2".to_string(),
        },
        ..AppConfig::default()
    }
}

pub fn normalize_provider_type(provider_type: &str) -> String {
    match provider_type.trim().to_ascii_lowercase().as_str() {
        "google" | "google-ai" => "gemini".to_string(),
        "localai" => "ollama".to_string(),
        other => other.to_string(),
    }
}

/// Serialize `AppConfig` to TOML text.
pub fn config_to_toml_wrapped(config: &AppConfig) -> CliResult<String> {
    antikythera_core::config::toml_config::config_to_toml(config).map_err(CliError::Config)
}

/// Deserialize `AppConfig` from TOML text.
pub fn config_from_toml_wrapped(data: &str) -> CliResult<AppConfig> {
    antikythera_core::config::toml_config::config_from_toml(data).map_err(CliError::Config)
}

/// Load `AppConfig` from `path` (defaults to [`CONFIG_PATH`] = `app.toml`).
pub fn load_app_config(path: Option<&Path>) -> CliResult<AppConfig> {
    let config_path = path.unwrap_or(Path::new(CONFIG_PATH));
    if !config_path.exists() {
        return Err(CliError::Config(format!(
            "Config not found: {}",
            config_path.display()
        )));
    }
    let data = std::fs::read_to_string(config_path)?;
    config_from_toml_wrapped(&data)
}

/// Save `AppConfig` to `path` (defaults to [`CONFIG_PATH`] = `app.toml`).
pub fn save_app_config(config: &AppConfig, path: Option<&Path>) -> CliResult<()> {
    let config_path = path.unwrap_or(Path::new(CONFIG_PATH));
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = config_to_toml_wrapped(config)?;
    std::fs::write(config_path, data.as_bytes())?;
    Ok(())
}

/// Returns `true` if the config file already exists at the default path.
pub fn config_exists() -> bool {
    Path::new(CONFIG_PATH).exists()
}

/// Create and persist a default `AppConfig` at [`CONFIG_PATH`].
pub fn init_default_config() -> CliResult<AppConfig> {
    let config = recommended_default_config();
    save_app_config(&config, None)?;
    Ok(config)
}
