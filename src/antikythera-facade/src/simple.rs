use crate::error::FacadeError;
use antikythera_core::application::agent::{Agent, AgentOptions, AgentOutcome};
use antikythera_core::application::client::{ClientConfig, McpClient};
use antikythera_core::infrastructure::model::DynamicModelProvider;
use antikythera_core::infrastructure::model::traits::ModelClient;
use std::sync::Arc;

/// Agent sederhana — entry point tercepat untuk membangun AI agent.
///
/// # Contoh Penggunaan
///
/// ```no_run
/// use antikythera_facade::SimpleAgent;
///
/// #[tokio::main]
/// async fn main() {
///     let mut agent = SimpleAgent::ollama("gpt-oss:120b-cloud").await.unwrap();
///     let response = agent.chat("Who are you?").await.unwrap();
///     println!("{response}");
/// }
/// ```
pub struct SimpleAgent {
    agent: Agent<DynamicModelProvider>,
    session_id: Option<String>,
}

impl SimpleAgent {
    /// Quick-start dengan Ollama provider (default endpoint: http://127.0.0.1:11434).
    pub async fn ollama(model: &str) -> Result<Self, FacadeError> {
        #[cfg(feature = "ollama")]
        {
            let provider = antikythera_provider_ollama::OllamaClient::new(model);
            Self::from_client("ollama", model, Box::new(provider))
        }
        #[cfg(not(feature = "ollama"))]
        {
            Err(FacadeError::ProviderNotAvailable("ollama".into()))
        }
    }

    /// Quick-start dengan Ollama provider pada custom endpoint.
    pub async fn ollama_with_endpoint(endpoint: &str, model: &str) -> Result<Self, FacadeError> {
        #[cfg(feature = "ollama")]
        {
            let provider =
                antikythera_provider_ollama::OllamaClient::with_endpoint(endpoint, model);
            Self::from_client("ollama", model, Box::new(provider))
        }
        #[cfg(not(feature = "ollama"))]
        {
            Err(FacadeError::ProviderNotAvailable("ollama".into()))
        }
    }

    /// Quick-start dengan OpenAI provider.
    pub fn openai(api_key: &str, model: &str) -> Result<Self, FacadeError> {
        #[cfg(feature = "openai")]
        {
            let provider = antikythera_provider_openai::OpenAiClient::new(api_key, model)?;
            Self::from_client("openai", model, Box::new(provider))
        }
        #[cfg(not(feature = "openai"))]
        {
            let _ = (api_key, model);
            Err(FacadeError::ProviderNotAvailable("openai".into()))
        }
    }

    /// Quick-start dengan Gemini provider.
    pub fn gemini(api_key: &str, model: &str) -> Result<Self, FacadeError> {
        #[cfg(feature = "gemini")]
        {
            let provider = antikythera_provider_gemini::GeminiClient::new(api_key, model)?;
            Self::from_client("gemini", model, Box::new(provider))
        }
        #[cfg(not(feature = "gemini"))]
        {
            let _ = (api_key, model);
            Err(FacadeError::ProviderNotAvailable("gemini".into()))
        }
    }

    /// Chat single-turn — kirim prompt, dapat response.
    pub async fn chat(&mut self, prompt: &str) -> Result<String, FacadeError> {
        let options = if let Some(ref sid) = self.session_id {
            AgentOptions {
                session_id: Some(sid.clone()),
                ..Default::default()
            }
        } else {
            AgentOptions::default()
        };
        let outcome = self
            .agent
            .run(prompt.to_string(), options)
            .await
            .map_err(FacadeError::Agent)?;
        Ok(extract_text(&outcome.response))
    }

    /// Chat dengan tool-use — full agent loop dengan max_steps.
    pub async fn chat_with_tools(&mut self, prompt: &str) -> Result<AgentOutcome, FacadeError> {
        let options = AgentOptions {
            max_steps: 10,
            ..Default::default()
        };
        self.agent
            .run(prompt.to_string(), options)
            .await
            .map_err(FacadeError::Agent)
    }

    /// Chat dengan custom options.
    pub async fn chat_with_options(
        &mut self,
        prompt: &str,
        options: AgentOptions,
    ) -> Result<AgentOutcome, FacadeError> {
        self.agent
            .run(prompt.to_string(), options)
            .await
            .map_err(FacadeError::Agent)
    }

    /// Set session ID untuk kontinuitas percakapan.
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    fn from_client(
        provider_name: &str,
        model: &str,
        client: Box<dyn ModelClient>,
    ) -> Result<Self, FacadeError> {
        let provider =
            DynamicModelProvider::new().register(provider_name, vec![model.to_string()], client);
        let config = ClientConfig::new(provider_name, model);
        let mcp_client = McpClient::new(provider, config, None);
        let agent = Agent::new(Arc::new(mcp_client));
        Ok(Self {
            agent,
            session_id: None,
        })
    }
}

fn extract_text(response: &serde_json::Value) -> String {
    match response {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(map) => map
            .get("content")
            .or_else(|| map.get("text"))
            .or_else(|| map.get("response"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        _ => response.to_string(),
    }
}
