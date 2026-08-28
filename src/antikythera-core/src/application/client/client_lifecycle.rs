use super::{ClientConfigSnapshot, McpClient};
use crate::application::config::PromptsConfig;
use crate::infrastructure::model::ModelProvider;

impl<P: ModelProvider> McpClient<P> {
    /// Return the default provider identifier (e.g., `"gemini"`, `"openai"`).
    pub fn default_provider(&self) -> &str {
        &self.config.default_provider
    }

    /// Return the default model name used when no per-request override is set.
    pub fn default_model(&self) -> &str {
        &self.config.default_model
    }

    /// Build a [`ClientConfigSnapshot`] from the current config for display layers.
    ///
    /// The snapshot includes the raw TOML representation used by the Settings overlay.
    pub fn config_snapshot(&self) -> ClientConfigSnapshot {
        let app_config = self.config.to_app_config();
        let prompt_template = app_config.prompt_template().to_string();
        let raw = app_config.to_raw_toml();
        ClientConfigSnapshot {
            model: app_config.model_name().to_string(),
            default_provider: app_config.default_provider().to_string(),
            system_prompt: app_config.system_prompt.clone(),
            prompt_template,
            tools: app_config.tools.clone(),
            servers: app_config.servers.clone(),
            raw,
        }
    }

    /// Return the prompts configuration section (system prompt templates, overrides).
    pub fn prompts(&self) -> &PromptsConfig {
        &self.config.prompts
    }

    /// Update session stats based on agent execution outcome.
    pub async fn record_agent_outcome(
        &self,
        session_id: &str,
        steps: &[crate::application::agent::AgentStep],
    ) {
        let sessions = self.sessions.lock().await;
        let manager = sessions.manager();

        for (i, step) in steps.iter().enumerate() {
            let _ = manager.record_tool(session_id, &step.tool, (i + 1) as u32);
        }
    }
}
