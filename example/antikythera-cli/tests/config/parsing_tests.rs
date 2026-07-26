// Config parsing tests - testing TOML serialization and provider conversion.
//
// The application stores all configuration as a single TOML file (app.toml).
// These tests verify that the TOML roundtrip preserves field values correctly
// and that CLI provider type helpers work as expected.

use antikythera_cli::config::{ModelInfo as ConfigModelInfo, ProviderConfig};
use antikythera_cli::infrastructure::llm::{ModelProviderConfig, providers_from_config};
use antikythera_core::config::AppConfig;
use antikythera_core::config::toml_config::{TomlAppConfig, config_to_toml};
use antikythera_core::config::ModelConfig;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Serialize a `TomlAppConfig` to a temp file and return the path.
fn write_toml_config(dir: &Path, config: &TomlAppConfig) -> std::path::PathBuf {
    let path = dir.join("app.toml");
    let data = config_to_toml(config).expect("Failed to serialize TomlAppConfig");
    fs::write(&path, data.as_bytes()).expect("Failed to write app.toml");
    path
}

/// A minimal valid `TomlAppConfig` with sensible defaults for testing.
fn minimal_toml_config() -> TomlAppConfig {
    TomlAppConfig {
        model: ModelConfig {
            default_provider: "gemini".to_string(),
            model: "gemini-1.5-flash".to_string(),
        },
        ..Default::default()
    }
}

// Split into parts for consistent test organization.
include!("parsing_tests/minimal_valid_config.rs");
include!("parsing_tests/provider_type_detection.rs");
include!("parsing_tests/prompt_template_parse.rs");
