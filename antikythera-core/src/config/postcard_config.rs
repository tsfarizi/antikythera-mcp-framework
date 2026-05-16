//! Unified Postcard-based Configuration
//!
//! All configuration is stored as a single Postcard binary file (`app.pc`).
//! CLI and FFI provide full access to all config fields.
//!
//! ## Naming disambiguation
//!
//! Several type names in this module (e.g. `AppConfig`, `ServerConfig`) are
//! intentionally different from the runtime types in [`super::app`] even though
//! they serve related purposes:
//!
//! | This module (`postcard_config`) | Runtime module (`app`) | Purpose |
//! |---------------------------------|------------------------|---------|
//! | [`PostcardAppConfig`] (`AppConfig` alias) | [`super::app::AppConfig`] | Serialised blob ↔ runtime struct |
//! | [`PostcardServerConfig`] (`ServerConfig` alias) | [`super::app::RestServerConfig`] | REST server bind settings |
//! | [`AgentConfig`] | *(derived at runtime)* | Agent tuning knobs |
//!
//! **Use [`PostcardAppConfig`]** when you need to disambiguate the serialised
//! form from the runtime form in the same scope (e.g. in `loader.rs` or wizard
//! code).  `AppConfig` and `PostcardAppConfig` are the **same type**.
//!
//! The canonical source of truth for native execution is
//! [`super::app::AppConfig`], which is produced by
//! [`super::loader::load_config`] from the Postcard blob.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// Import security configuration types
use crate::security::config::SecurityConfig;

// ============================================================================
// Unified Configuration Structure
// ============================================================================

/// Complete application configuration (single Postcard blob)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostcardAppConfig {
    /// REST server settings
    pub server: PostcardServerConfig,
    /// LLM providers
    pub providers: Vec<ProviderConfig>,
    /// Default provider and model
    pub model: ModelConfig,
    /// All prompt templates
    pub prompts: PromptsConfig,
    /// Agent behavior settings
    pub agent: AgentConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Custom key-value pairs for extensibility
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

// ============================================================================
// Server Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostcardServerConfig {
    /// Bind address (e.g., "127.0.0.1:8080")
    pub bind: String,
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
    /// API documentation servers
    pub docs: Vec<DocServerConfig>,
}

impl Default for PostcardServerConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:8080".to_string(),
            cors_origins: Vec::new(),
            docs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocServerConfig {
    pub url: String,
    pub description: String,
}

// ============================================================================
// Provider Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Unique provider ID
    pub id: String,
    /// Provider type (openai, anthropic, ollama, gemini, etc.)
    pub provider_type: String,
    /// API endpoint URL
    pub endpoint: String,
    /// API key reference (env var name)
    pub api_key: String,
    /// Available models
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub display_name: String,
}

// ============================================================================
// Model Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Default provider ID
    pub default_provider: String,
    /// Default model name
    pub model: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            default_provider: "ollama".to_string(),
            model: "llama3".to_string(),
        }
    }
}

// ============================================================================
// Prompts Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsConfig {
    pub template: String,
    pub tool_guidance: String,
    pub fallback_guidance: String,
    pub json_retry_message: String,
    pub tool_result_instruction: String,
    pub agent_instructions: String,
    pub language_instructions: String,
    pub agent_max_steps_error: String,
    pub no_tools_guidance: String,
    /// Field names probed in fallback when the model returns an unknown action
    pub fallback_response_keys: Vec<String>,
}

impl Default for PromptsConfig {
    fn default() -> Self {
        use super::app::PromptsConfig as AppPrompts;
        Self {
            template: AppPrompts::default_template().to_string(),
            tool_guidance: AppPrompts::default_tool_guidance().to_string(),
            fallback_guidance: AppPrompts::default_fallback_guidance().to_string(),
            json_retry_message: AppPrompts::default_json_retry_message().to_string(),
            tool_result_instruction: AppPrompts::default_tool_result_instruction().to_string(),
            agent_instructions: AppPrompts::default_agent_instructions().to_string(),
            language_instructions: AppPrompts::default_language_instructions().to_string(),
            agent_max_steps_error: AppPrompts::default_agent_max_steps_error().to_string(),
            no_tools_guidance: AppPrompts::default_no_tools_guidance().to_string(),
            fallback_response_keys: AppPrompts::default_fallback_response_keys()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

impl PromptsConfig {
    /// Default prompt template (delegates to runtime default)
    pub fn default_template() -> &'static str {
        super::app::PromptsConfig::default_template()
    }
}

impl From<super::app::PromptsConfig> for PromptsConfig {
    fn from(app: super::app::PromptsConfig) -> Self {
        Self {
            template: app.template.unwrap_or_default(),
            tool_guidance: app.tool_guidance.unwrap_or_default(),
            fallback_guidance: app.fallback_guidance.unwrap_or_default(),
            json_retry_message: app.json_retry_message.unwrap_or_default(),
            tool_result_instruction: app.tool_result_instruction.unwrap_or_default(),
            agent_instructions: app.agent_instructions.unwrap_or_default(),
            language_instructions: app.language_instructions.unwrap_or_default(),
            agent_max_steps_error: app.agent_max_steps_error.unwrap_or_default(),
            no_tools_guidance: app.no_tools_guidance.unwrap_or_default(),
            fallback_response_keys: app.fallback_response_keys.unwrap_or_default(),
        }
    }
}

impl From<PromptsConfig> for super::app::PromptsConfig {
    fn from(pc: PromptsConfig) -> Self {
        Self {
            template: opt_nonempty(pc.template),
            tool_guidance: opt_nonempty(pc.tool_guidance),
            fallback_guidance: opt_nonempty(pc.fallback_guidance),
            json_retry_message: opt_nonempty(pc.json_retry_message),
            tool_result_instruction: opt_nonempty(pc.tool_result_instruction),
            agent_instructions: opt_nonempty(pc.agent_instructions),
            language_instructions: opt_nonempty(pc.language_instructions),
            agent_max_steps_error: opt_nonempty(pc.agent_max_steps_error),
            no_tools_guidance: opt_nonempty(pc.no_tools_guidance),
            fallback_response_keys: if pc.fallback_response_keys.is_empty() {
                None
            } else {
                Some(pc.fallback_response_keys)
            },
        }
    }
}

fn opt_nonempty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

// ============================================================================
// Agent Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum tool interaction steps
    pub max_steps: u32,
    /// Verbose logging
    pub verbose: bool,
    /// Auto-execute tools
    pub auto_execute_tools: bool,
    /// Session timeout (seconds)
    pub session_timeout_secs: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_steps: 10,
            verbose: false,
            auto_execute_tools: true,
            session_timeout_secs: 300,
        }
    }
}

// ============================================================================
// Postcard Serialization
// ============================================================================

/// Configuration file path (project root)
pub const CONFIG_PATH: &str = "app.pc";

/// Serialize configuration to Postcard binary
pub fn config_to_postcard(config: &PostcardAppConfig) -> Result<Vec<u8>, String> {
    postcard::to_allocvec(config).map_err(|e| format!("Failed to serialize config: {}", e))
}

/// Deserialize configuration from Postcard binary
pub fn config_from_postcard(data: &[u8]) -> Result<PostcardAppConfig, String> {
    postcard::from_bytes(data).map_err(|e| format!("Failed to deserialize config: {}", e))
}

/// Load configuration from file
pub fn load_config(path: Option<&Path>) -> Result<PostcardAppConfig, String> {
    let config_path = path.unwrap_or(Path::new(CONFIG_PATH));

    if !config_path.exists() {
        return Err(format!("Config file not found: {}", config_path.display()));
    }

    let data =
        std::fs::read(config_path).map_err(|e| format!("Failed to read config file: {}", e))?;

    config_from_postcard(&data)
}

/// Alias for [`PostcardAppConfig`] that makes the distinction from
/// [`super::app::AppConfig`] explicit in code that imports both.
///
/// ```rust,ignore
/// use antikythera_core::config::postcard_config::PostcardAppConfig;
/// use antikythera_core::config::AppConfig; // runtime form
/// ```
pub type AppConfig = PostcardAppConfig;

/// Backwards-compatible alias for serialized server config.
pub type ServerConfig = PostcardServerConfig;

/// Save configuration to file
pub fn save_config(config: &PostcardAppConfig, path: Option<&Path>) -> Result<(), String> {
    let config_path = path.unwrap_or(Path::new(CONFIG_PATH));

    // Create directory if needed
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let data = config_to_postcard(config)?;

    std::fs::write(config_path, &data)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}
