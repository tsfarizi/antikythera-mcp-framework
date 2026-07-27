//! Standalone Configuration Management Binary
//!
//! Backward-compatible binary for managing `app.toml`.
//! Prefer using `antikythera config <subcmd>` for new workflows.

use antikythera_cli::cli::ConfigCommand;
use antikythera_cli::error::CliResult;
use antikythera_log::{cli_eprint, cli_print};
use clap::Parser;

#[derive(Parser)]
#[command(name = "antikythera-config")]
#[command(about = "Manage Antikythera configuration (app.toml)")]
struct ConfigCli {
    #[command(subcommand)]
    command: ConfigCommand,
}

fn main() {
    let args = ConfigCli::parse();
    if let Err(e) = execute_config_cli(args.command) {
        cli_eprint!("Error: {}", e);
        std::process::exit(1);
    }
}

pub fn execute_config_cli(command: ConfigCommand) -> CliResult<()> {
    use antikythera_cli::config::{
        config_exists, get_field, init_default_config, load_app_config, normalize_provider_type,
        save_app_config, set_field, AppConfig, ModelInfo, ProviderConfig, CONFIG_PATH,
    };

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
            let value = get_field(&cfg, &field)?;
            cli_print!("{}", value);
            Ok(())
        }

        ConfigCommand::Set { field, value } => {
            let mut cfg = load_app_config(None)?;
            set_field(&mut cfg, &field, &value)?;
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
                return Err(antikythera_cli::error::CliError::Validation(format!(
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
                Err(antikythera_cli::error::CliError::Validation(format!(
                    "Provider '{}' not found",
                    id
                )))
            } else {
                save_app_config(&cfg, None)?;
                cli_print!("Provider '{}' removed", id);
                Ok(())
            }
        }

        ConfigCommand::SetModel { provider, model } => {
            let mut cfg = load_app_config(None)?;

            if !cfg.providers.iter().any(|p| p.id == provider) {
                return Err(antikythera_cli::error::CliError::Validation(format!(
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
                return Err(antikythera_cli::error::CliError::Validation(format!(
                    "Provider '{}' not found",
                    provider
                )));
            };
            if p.models.iter().any(|m| m.name == model) {
                return Err(antikythera_cli::error::CliError::Validation(format!(
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
                return Err(antikythera_cli::error::CliError::Validation(format!(
                    "Provider '{}' not found",
                    provider
                )));
            };
            let before = p.models.len();
            p.models.retain(|m| m.name != model);
            if p.models.len() == before {
                return Err(antikythera_cli::error::CliError::Validation(format!(
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
