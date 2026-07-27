use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "antikythera",
    version,
    about = "Antikythera MCP client — interactive TUI with configurable model providers"
)]
pub struct Cli {
    // ------------------------------------------------------------------
    // Global flags (apply to all subcommands including default TUI)
    // ------------------------------------------------------------------
    #[arg(long, default_value = "http://127.0.0.1:11434")]
    pub ollama_url: String,

    /// Override the active provider ID without editing app.toml.
    #[arg(long)]
    pub provider: Option<String>,

    /// Override the active model name without editing app.toml.
    #[arg(long)]
    pub model: Option<String>,

    /// Override the endpoint for the selected provider.
    #[arg(long)]
    pub provider_endpoint: Option<String>,

    /// Path to app.toml configuration file.
    #[arg(long)]
    pub config: Option<String>,

    /// Override the system prompt.
    #[arg(long)]
    pub system: Option<String>,

    /// Enable token streaming output to terminal.
    #[arg(long)]
    pub stream: bool,

    /// Enable session storage initialization from app.toml `[storage]` section.
    #[arg(long)]
    pub storage: bool,

    // ------------------------------------------------------------------
    // Multi-agent flags (used with --multi-agent, feature-gated)
    // ------------------------------------------------------------------
    /// Run in multi-agent orchestrator mode (requires `multi-agent` feature).
    #[arg(long)]
    pub multi_agent: bool,

    /// Path to a JSON file containing agent profile definitions.
    #[arg(long)]
    pub agents: Option<String>,

    /// Task prompt to dispatch in multi-agent mode.
    #[arg(long)]
    pub task: Option<String>,

    /// Target a specific agent by ID (uses DirectRouter).
    #[arg(long)]
    pub target_agent: Option<String>,

    /// Execution mode for the multi-agent orchestrator.
    #[arg(long, default_value = "auto")]
    pub execution_mode: String,

    // ------------------------------------------------------------------
    // Subcommands
    // ------------------------------------------------------------------
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Manage application configuration (app.toml)
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },

    /// WASM host-FFI diagnostic probe
    WasmHarness {
        /// Path to WASM module to probe.
        #[arg(long)]
        wasm: Option<String>,

        /// Task description for the probe.
        #[arg(long)]
        task: Option<String>,

        /// Stub LLM response returned by host callback.
        #[arg(long)]
        wasm_llm_response: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommand {
    /// Initialize default configuration
    Init,
    /// Show all configuration as JSON
    Show,
    /// Get a specific field value
    Get { field: String },
    /// Set a specific field value
    Set { field: String, value: String },
    /// Add a provider
    AddProvider {
        id: String,
        #[arg(name = "type")]
        provider_type: String,
        endpoint: String,
        /// API key environment-variable name (e.g. GEMINI_API_KEY). Omit for Ollama.
        #[arg(name = "api_key")]
        api_key: Option<String>,
    },
    /// Remove a provider by ID
    RemoveProvider { id: String },
    /// Set the default provider and model
    SetModel { provider: String, model: String },
    /// Set the REST server bind address
    SetBind { address: String },
    /// Export configuration as JSON
    Export { output: Option<String> },
    /// Import configuration from JSON
    Import { input: String },
    /// Reset to default configuration
    Reset,
    /// Show config status
    Status,
    /// Add a model to a provider's model list
    AddModel {
        provider: String,
        model: String,
        /// Optional human-readable display name
        #[arg(name = "display_name")]
        display_name: Option<String>,
    },
    /// Remove a model from a provider's model list
    RemoveModel {
        provider: String,
        model: String,
    },
}
