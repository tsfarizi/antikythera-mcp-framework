//! Main CLI Binary Entry Point
//!
//! Standalone TUI application with subcommands:
//!
//! | Command | Description |
//! |:--------|:------------|
//! | *(default)* | Interactive ratatui TUI chat session |
//! | `config` | Manage app.toml configuration |
//! | `wasm-harness` | Host-FFI WASM probe for runtime/session/tool validation |

use std::path::Path;
use std::sync::Arc;

use antikythera_cli::cli::{Cli, Command, ConfigCommand};
use antikythera_cli::config::{
    init_default_config, load_app_config, save_app_config, normalize_provider_type,
    config_exists, get_field as config_get_field, set_field as config_set_field,
    AppConfig, ProviderConfig, ModelInfo, CONFIG_PATH,
};
use antikythera_cli::storage;
use antikythera_cli::error::{CliError, CliResult};
use antikythera_cli::domain::use_cases::{render_wasm_stream_report, run_wasm_stream_probe};
use antikythera_cli::infrastructure::llm::install_terminal_stream_sink;
use antikythera_cli::infrastructure::llm::providers_from_config;
use antikythera_cli::presentation::tui;
use antikythera_cli::runtime::{build_runtime_client, materialize_runtime_config};
use antikythera_core::infrastructure::model::DynamicModelProvider;
use antikythera_core::AppConfig as CoreAppConfig;
use antikythera_core::application::client::McpClient;
use antikythera_log::{cli_eprint, cli_print};
use clap::Parser;

#[cfg(feature = "multi-agent")]
use antikythera_core::application::agent::multi_agent::task::AgentTask;

#[cfg(feature = "multi-agent")]
use antikythera_core::application::agent::multi_agent::{
    AgentProfile, DirectRouter, ExecutionMode, MultiAgentOrchestrator, RoundRobinRouter,
};

#[cfg(feature = "multi-agent")]
use antikythera_core::application::agent::multi_agent::budget::OrchestratorBudget;

#[cfg(feature = "multi-agent")]
use antikythera_core::application::agent::multi_agent::guardrails::{
    BudgetGuardrail, GuardrailChain, TimeoutGuardrail,
};

fn load_cli_env() {
    let manifest_env = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    if manifest_env.exists() {
        let _ = dotenvy::from_filename(&manifest_env);
        return;
    }
    let _ = dotenvy::dotenv();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_cli_env();

    let cli = Cli::parse();

    match cli.command {
        // ----------------------------------------------------------
        // Subcommand: config
        // ----------------------------------------------------------
        Some(Command::Config { command }) => {
            execute_config_command(command)?;
        }

        // ----------------------------------------------------------
        // Subcommand: wasm-harness
        // ----------------------------------------------------------
        Some(Command::WasmHarness { wasm, task, wasm_llm_response }) => {
            run_wasm_harness(cli.stream, wasm, task, wasm_llm_response).await?;
        }

        // ----------------------------------------------------------
        // Default: launch interactive TUI
        // ----------------------------------------------------------
        None => {
            run_tui(cli).await?;
        }
    }

    Ok(())
}

// =========================================================================
// TUI (default mode)
// =========================================================================

async fn run_tui(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = cli.config.as_deref().map(Path::new);
    let config = CoreAppConfig::load(config_path)?;

    if cli.storage {
        let storage_config = storage::load_storage_config(config_path)?;
        storage::print_storage_status(&storage_config);
        let _engine = storage::init_storage(config_path).await?;
        cli_eprint!("[storage] engine initialized successfully");
    }

    let initial_providers = providers_from_config(&config.providers);

    let provider_override = cli.provider.clone().or_else(|| {
        let p = config.model.default_provider.trim().to_string();
        if p.is_empty() { None } else { Some(p) }
    });
    let model_override = cli.model.clone().or_else(|| {
        let m = config.model.model.trim().to_string();
        if m.is_empty() { None } else { Some(m) }
    });
    let system_override = cli
        .system
        .clone()
        .or_else(|| config.custom.get("system_prompt").cloned())
        .or_else(|| config.system_prompt.clone());

    let (runtime_config, providers) = materialize_runtime_config(
        &config,
        &initial_providers,
        provider_override.as_deref(),
        model_override.as_deref(),
        cli.provider_endpoint.as_deref(),
        Some(cli.ollama_url.as_str()),
        system_override.as_deref(),
    )?;

    if cli.stream {
        install_terminal_stream_sink();
    }

    if cli.multi_agent {
        #[cfg(feature = "multi-agent")]
        {
            let client = build_runtime_client(
                &runtime_config,
                &providers,
                std::collections::HashMap::new(),
            )?;
            run_multi_agent(cli, client).await?;
        }
        #[cfg(not(feature = "multi-agent"))]
        {
            return Err("multi-agent feature is not enabled in this build.\n\
                 Rebuild with: cargo build --features multi-agent"
                .into());
        }
    } else {
        tui::run_chat_app(runtime_config, providers).await?;
    }

    Ok(())
}

// =========================================================================
// WASM Harness subcommand
// =========================================================================

async fn run_wasm_harness(
    stream: bool,
    wasm: Option<String>,
    task: Option<String>,
    wasm_llm_response: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = wasm
        .unwrap_or_else(|| "target/wasm32-wasip1/release/antikythera_sdk.wasm".to_string());

    let task_input = task
        .unwrap_or_else(|| "WASM harness smoke test".to_string());

    let default_response = r#"{"content":"harness-ok","model":"wasm-harness"}"#.to_string();
    let llm_payload = wasm_llm_response.unwrap_or(default_response);

    if !stream {
        cli_eprint!("[wasm-harness] enabling stream diagnostics for dev tooling output");
    }
    let stream_report = run_wasm_stream_probe(&task_input, &llm_payload, true)?;

    cli_print!("== WASM Host FFI Harness ==");
    cli_print!("artifact: {}", wasm_path);
    cli_print!("mode: ffi-host-probe");
    cli_print!();
    cli_print!("{}", render_wasm_stream_report(&stream_report)?);

    cli_print!("\n== WASM Dev Summary JSON ==");
    cli_print!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "artifact": wasm_path,
            "ffi_stream_probe": stream_report,
        }))?
    );

    Ok(())
}

// =========================================================================
// Multi-agent orchestrator (feature-gated)
// =========================================================================

#[cfg(feature = "multi-agent")]
async fn run_multi_agent(
    cli: Cli,
    client: Arc<McpClient<DynamicModelProvider>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let exec_mode = ExecutionMode::from_spec(&cli.execution_mode).unwrap_or(ExecutionMode::Auto);

    let profiles: Vec<AgentProfile> = if let Some(agents_path) = cli.agents.as_deref() {
        let raw = std::fs::read_to_string(agents_path)
            .map_err(|e| format!("Failed to read agents file '{}': {e}", agents_path))?;
        serde_json::from_str(&raw).map_err(|e| format!("Failed to parse agents JSON: {e}"))?
    } else {
        vec![AgentProfile {
            id: "default".to_string(),
            name: "Default Agent".to_string(),
            role: "general".to_string(),
            system_prompt: None,
            max_steps: None,
        }]
    };

    let mut orch = MultiAgentOrchestrator::new(client, exec_mode);
    for profile in profiles {
        orch = orch.register_agent(profile);
    }

    if let Some(target) = cli.target_agent.as_deref() {
        let _target = target.to_string();
        let router = Arc::new(DirectRouter);
        orch = orch.with_router(router);
        cli_eprint!("Routing all tasks to agent: {target}");
    } else if orch.agent_count() > 1 {
        orch = orch.with_router(Arc::new(RoundRobinRouter::new()));
    }

    let budget = OrchestratorBudget::new()
        .with_max_concurrent_tasks(8)
        .with_max_total_steps(1_000);

    let guardrails = GuardrailChain::new()
        .with_guardrail(Arc::new(TimeoutGuardrail::new(300_000)))
        .with_guardrail(Arc::new(BudgetGuardrail::new().with_max_task_steps(50)));

    orch = orch.with_budget(budget).with_guardrails(guardrails);

    cli_eprint!(
        "Multi-agent orchestrator ready: {} agent(s), mode={}, guardrails={}",
        orch.agent_count(),
        exec_mode,
        orch.guardrail_count(),
    );

    let task_input = if let Some(t) = cli.task.as_deref() {
        t.to_string()
    } else {
        cli_eprint!("Reading task from stdin (send EOF when done)...");
        let mut buf = String::new();
        {
            use std::io::Read;
            std::io::stdin().read_to_string(&mut buf)?;
        }
        buf.trim().to_string()
    };

    if task_input.is_empty() {
        return Err("No task input provided. Use --task <text> or pipe to stdin.".into());
    }

    let task = AgentTask::new(task_input);
    let result = orch.dispatch(task).await;

    cli_print!("{}", serde_json::to_string_pretty(&result)?);

    if !result.success {
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(not(feature = "multi-agent"))]
async fn run_multi_agent(
    _cli: Cli,
    _client: Arc<McpClient<DynamicModelProvider>>,
) -> Result<(), Box<dyn std::error::Error>> {
    Err("multi-agent feature is not enabled in this build.\n\
         Rebuild with: cargo build --features multi-agent"
        .into())
}

// =========================================================================
// Config subcommand handler
// =========================================================================

fn execute_config_command(command: ConfigCommand) -> CliResult<()> {
    match command {
        ConfigCommand::Init => {
            if config_exists() {
                cli_print!("Configuration already exists at: {}", CONFIG_PATH);
                cli_print!("Use 'reset' to overwrite.");
                return Ok(());
            }
            let cfg = init_default_config()?;
            cli_print!("Default configuration created at: {}", CONFIG_PATH);
            cli_print!("  Providers: {}", cfg.providers.len());
            for provider in &cfg.providers {
                cli_print!(
                    "  - {} [{}] -> {}",
                    provider.id,
                    provider.provider_type,
                    provider.endpoint
                );
            }
            Ok(())
        }

        ConfigCommand::Show => {
            let cfg = load_app_config(None)?;
            let json = serde_json::to_string_pretty(&cfg)?;
            cli_print!("{}", json);
            Ok(())
        }

        ConfigCommand::Get { field } => {
            let cfg = load_app_config(None)?;
            let value = config_get_field(&cfg, &field)?;
            cli_print!("{}", value);
            Ok(())
        }

        ConfigCommand::Set { field, value } => {
            let mut cfg = load_app_config(None)?;
            config_set_field(&mut cfg, &field, &value)?;
            save_app_config(&cfg, None)?;
            cli_print!("Set '{}' = '{}'", field, value);
            Ok(())
        }

        ConfigCommand::AddProvider {
            id,
            provider_type,
            endpoint,
            api_key,
        } => {
            let mut cfg = load_app_config(None)?;

            if cfg.providers.iter().any(|p| p.id == id) {
                return Err(CliError::Validation(format!(
                    "Provider '{}' already exists",
                    id
                )));
            }

            let provider_type = normalize_provider_type(&provider_type);
            cfg.providers.push(ProviderConfig {
                id: id.clone(),
                provider_type,
                endpoint,
                api_key: api_key.unwrap_or_default(),
                models: vec![],
            });

            save_app_config(&cfg, None)?;
            cli_print!("Provider '{}' added", id);
            Ok(())
        }

        ConfigCommand::RemoveProvider { id } => {
            let mut cfg = load_app_config(None)?;
            let initial_len = cfg.providers.len();
            cfg.providers.retain(|p| p.id != id);

            if cfg.providers.len() == initial_len {
                Err(CliError::Validation(format!("Provider '{}' not found", id)))
            } else {
                save_app_config(&cfg, None)?;
                cli_print!("Provider '{}' removed", id);
                Ok(())
            }
        }

        ConfigCommand::SetModel { provider, model } => {
            let mut cfg = load_app_config(None)?;

            if !cfg.providers.iter().any(|p| p.id == provider) {
                return Err(CliError::Validation(format!(
                    "Provider '{}' not found",
                    provider
                )));
            }

            cfg.model.default_provider = provider.clone();
            cfg.model.model = model.clone();

            save_app_config(&cfg, None)?;
            cli_print!("Default model set: {} / {}", provider, model);
            Ok(())
        }

        ConfigCommand::SetBind { address } => {
            let mut cfg = load_app_config(None)?;
            cfg.server.bind = address.clone();
            save_app_config(&cfg, None)?;
            cli_print!("Bind address set to: {}", address);
            Ok(())
        }

        ConfigCommand::Export { output } => {
            let cfg = load_app_config(None)?;
            let json = serde_json::to_string_pretty(&cfg)?;

            match output {
                Some(path) => {
                    std::fs::write(&path, &json)?;
                    cli_print!("Exported to: {}", path);
                }
                None => cli_print!("{}", json),
            }
            Ok(())
        }

        ConfigCommand::Import { input } => {
            let json = std::fs::read_to_string(&input)?;
            let cfg: AppConfig = serde_json::from_str(&json)?;
            save_app_config(&cfg, None)?;
            cli_print!("Imported from: {}", input);
            Ok(())
        }

        ConfigCommand::Reset => {
            init_default_config()?;
            cli_print!("Configuration reset to defaults");
            cli_print!("  Path: {}", CONFIG_PATH);
            Ok(())
        }

        ConfigCommand::Status => {
            if config_exists() {
                let cfg = load_app_config(None)?;
                cli_print!("Config exists at: {}", CONFIG_PATH);
                cli_print!("  Providers: {}", cfg.providers.len());
                cli_print!(
                    "  Default: {}/{}",
                    cfg.model.default_provider,
                    cfg.model.model
                );
                cli_print!("  Server: {}", cfg.server.bind);
            } else {
                cli_print!("No config found at: {}", CONFIG_PATH);
                cli_print!("  Run 'init' to create default config.");
            }
            Ok(())
        }

        ConfigCommand::AddModel {
            provider,
            model,
            display_name,
        } => {
            let mut cfg = load_app_config(None)?;
            let Some(p) = cfg.providers.iter_mut().find(|p| p.id == provider) else {
                return Err(CliError::Validation(format!(
                    "Provider '{}' not found",
                    provider
                )));
            };
            if p.models.iter().any(|m| m.name == model) {
                return Err(CliError::Validation(format!(
                    "Model '{}' already exists in provider '{}'",
                    model, provider
                )));
            }
            p.models.push(ModelInfo {
                name: model.clone(),
                display_name: display_name.clone().unwrap_or_default(),
            });
            save_app_config(&cfg, None)?;
            cli_print!("Model '{}' added to provider '{}'", model, provider);
            Ok(())
        }

        ConfigCommand::RemoveModel { provider, model } => {
            let mut cfg = load_app_config(None)?;
            let Some(p) = cfg.providers.iter_mut().find(|p| p.id == provider) else {
                return Err(CliError::Validation(format!(
                    "Provider '{}' not found",
                    provider
                )));
            };
            let before = p.models.len();
            p.models.retain(|m| m.name != model);
            if p.models.len() == before {
                return Err(CliError::Validation(format!(
                    "Model '{}' not found in provider '{}'",
                    model, provider
                )));
            }
            save_app_config(&cfg, None)?;
            cli_print!("Model '{}' removed from provider '{}'", model, provider);
            Ok(())
        }
    }
}
