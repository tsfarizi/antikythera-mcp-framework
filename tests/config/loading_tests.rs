// Config loading tests - testing AppConfig::load behavior.
//
// The application stores all configuration as a single Postcard binary (app.pc).
// Tests verify: file-not-found error, self-heal on corrupt data, and correct
// field values on a valid binary.

use antikythera_core::config::postcard_config::{PostcardAppConfig, config_to_postcard};
use antikythera_core::config::ModelConfig;
use antikythera_core::config::{AppConfig, ConfigError};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Serialize a `PostcardAppConfig` to a temp file and return the path.
fn write_postcard_config(dir: &Path, config: &PostcardAppConfig) -> std::path::PathBuf {
    let path = dir.join("app.pc");
    let data = config_to_postcard(config).expect("Failed to serialize PostcardAppConfig");
    fs::write(&path, &data).expect("Failed to write app.pc");
    path
}

/// A minimal valid `PostcardAppConfig` for testing.
fn minimal_postcard_config() -> PostcardAppConfig {
    PostcardAppConfig {
        model: ModelConfig {
            default_provider: "test-provider".to_string(),
            model: "test-model".to_string(),
        },
        ..Default::default()
    }
}

// Split into 5 parts for consistent test organization.
include!("loading_tests/config_not_found.rs");
include!("loading_tests/self_heal_corrupt_data.rs");
include!("loading_tests/prompt_template_loading.rs");
include!("loading_tests/postcard_roundtrip.rs");
include!("loading_tests/actual_config_loading.rs");
