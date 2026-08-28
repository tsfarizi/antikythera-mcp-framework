//! Agent client trait and supporting types.
//!
//! The [`AgentClient`] trait abstracts the chat interface used by the agent
//! runner. Concrete implementations live in host application crates.

use crate::application::config::{AppConfig, PromptsConfig, ServerConfig, ToolConfig};
use crate::application::resilience::ContextWindowPolicy;
use crate::application::tooling::{ToolServerInterface, transport::McpTransport};
use crate::domain::types::ChatMessage;
use crate::domain::types::MessagePart;
use crate::infrastructure::model::{ModelError, ModelRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Error returned by [`AgentClient`] operations.
#[derive(Debug, Error)]
pub enum McpError {
    #[error(transparent)]
    Model(#[from] ModelError),
}

impl McpError {
    pub fn user_message(&self) -> String {
        match self {
            McpError::Model(err) => err.user_message(),
        }
    }
}

/// Request parameters for a chat interaction.
#[derive(Debug, Default)]
pub struct ChatRequest {
    /// The user's message/prompt
    pub prompt: String,
    /// Optional file/image attachments
    pub attachments: Vec<MessagePart>,
    /// Optional system prompt override
    pub system_prompt: Option<String>,
    /// Session ID for conversation continuity
    pub session_id: Option<String>,
    /// Raw mode - bypass all config system prompts and templates
    pub raw_mode: bool,
    /// Skip template composition - use system_prompt as-is
    pub bypass_template: bool,
    /// Force JSON mode - requests the LLM to output valid JSON
    pub force_json: bool,
}

/// Result from a chat interaction.
#[derive(Debug, Clone)]
pub struct ChatResult {
    /// The model's response content
    pub content: String,
    /// Session ID for this conversation
    pub session_id: String,
    /// Provider used for this request
    pub provider: String,
    /// Model used for this request
    pub model: String,
    /// Debug/execution logs
    pub logs: Vec<String>,
}

/// Prepared host-facing model request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedChatTurn {
    pub session_id: String,
    pub provider: String,
    pub model: String,
    pub model_request: ModelRequest,
    pub user_message: ChatMessage,
    pub logs: Vec<String>,
}

/// Client configuration for the MCP client.
#[derive(Clone)]
pub struct ClientConfig {
    /// Default provider ID to use
    pub default_provider: String,
    /// Default model name
    pub default_model: String,
    /// Optional system prompt override
    pub default_system_prompt: Option<String>,
    /// Available tools from MCP servers
    pub tools: Vec<ToolConfig>,
    /// MCP server configurations
    pub servers: Vec<ServerConfig>,
    /// Configurable prompts for agent behavior
    pub prompts: PromptsConfig,
    /// Pre-built builtin transports keyed by server name
    pub builtin_transports: HashMap<String, Arc<dyn McpTransport>>,
}

impl std::fmt::Debug for ClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientConfig")
            .field("default_provider", &self.default_provider)
            .field("default_model", &self.default_model)
            .field("default_system_prompt", &self.default_system_prompt)
            .field("tools", &self.tools)
            .field("servers", &self.servers)
            .field("prompts", &self.prompts)
            .field("builtin_transports_count", &self.builtin_transports.len())
            .finish()
    }
}

impl ClientConfig {
    /// Create a new client configuration with the specified provider and model.
    pub fn new(default_provider: impl Into<String>, default_model: impl Into<String>) -> Self {
        Self {
            default_provider: default_provider.into(),
            default_model: default_model.into(),
            default_system_prompt: None,
            tools: Vec::new(),
            servers: Vec::new(),
            prompts: PromptsConfig::default(),
            builtin_transports: HashMap::new(),
        }
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.default_system_prompt = Some(prompt.into());
        self
    }

    pub fn with_tools(mut self, tools: Vec<ToolConfig>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_servers(mut self, servers: Vec<ServerConfig>) -> Self {
        self.servers = servers;
        self
    }

    pub fn with_prompts(mut self, prompts: PromptsConfig) -> Self {
        self.prompts = prompts;
        self
    }

    pub fn with_builtin_transport(
        mut self,
        server_name: impl Into<String>,
        transport: Arc<dyn McpTransport>,
    ) -> Self {
        self.builtin_transports
            .insert(server_name.into(), transport);
        self
    }

    pub fn prompt_template(&self) -> &str {
        self.prompts.template()
    }

    pub fn to_app_config(&self) -> AppConfig {
        let mut config = AppConfig::default();
        config.set_default_provider(&self.default_provider);
        config.set_model(&self.default_model);
        config.system_prompt = self.default_system_prompt.clone();
        config.tools = self.tools.clone();
        config.servers = self.servers.clone();
        config.prompts = self.prompts.clone();
        config
    }
}

/// Read-only snapshot of the active client configuration.
#[derive(Debug, Clone)]
pub struct ClientConfigSnapshot {
    pub model: String,
    pub default_provider: String,
    pub system_prompt: Option<String>,
    pub prompt_template: String,
    pub tools: Vec<ToolConfig>,
    pub servers: Vec<ServerConfig>,
    /// Full TOML representation of the config
    pub raw: String,
}

/// Trait abstracting the chat interface used by the agent runner.
///
/// Implementors provide the concrete client that owns session history,
/// provider dispatch, and server connectivity.
pub trait AgentClient: Send + Sync {
    /// Return the list of registered tool configurations.
    fn tools(&self) -> &[ToolConfig];

    /// Return the default provider identifier.
    fn default_provider(&self) -> &str;

    /// Return the default model name.
    fn default_model(&self) -> &str;

    /// Return the prompts configuration section.
    fn prompts(&self) -> &PromptsConfig;

    /// Return a clone of the active tool server interface.
    fn server_bridge(&self) -> Arc<dyn ToolServerInterface>;

    /// Build a configuration snapshot for display layers.
    fn config_snapshot(&self) -> ClientConfigSnapshot;

    /// Send a chat request and return the result.
    #[allow(async_fn_in_trait)]
    async fn chat(&self, request: ChatRequest) -> Result<ChatResult, McpError>;

    /// Prune old messages from a session to fit within a context window policy.
    #[allow(async_fn_in_trait)]
    async fn prune_session(&self, session_id: &str, policy: &ContextWindowPolicy) -> usize;

    /// Record agent execution outcome in session stats.
    #[allow(async_fn_in_trait)]
    async fn record_agent_outcome(
        &self,
        session_id: &str,
        steps: &[crate::application::agent::AgentStep],
    );
}

/// Summarise text to a short preview string.
pub fn summarise_text(text: &str) -> String {
    const SNIPPET_LIMIT: usize = 160;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "(empty)".to_string();
    }
    let single_line = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut result = String::new();
    let mut chars = single_line.chars();
    for _ in 0..SNIPPET_LIMIT {
        if let Some(ch) = chars.next() {
            result.push(ch);
        } else {
            return result;
        }
    }
    if chars.next().is_some() {
        result.push('…');
    }
    result
}
