use thiserror::Error;

#[derive(Error, Debug)]
pub enum FacadeError {
    #[error(
        "No provider configured. Use SimpleAgent::ollama(), SimpleAgent::openai(), or SimpleAgent::gemini()"
    )]
    MissingProvider,

    #[error("Provider '{0}' not available. Enable the corresponding feature flag.")]
    ProviderNotAvailable(String),

    #[error("Agent error: {0}")]
    Agent(#[from] antikythera_core::application::agent::AgentError),

    #[error("Connection error: {0}")]
    Connection(String),
}
