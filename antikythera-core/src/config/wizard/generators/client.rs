//! Client configuration generator
//!
//! Generates and modifies app.toml containing:
//! - `[[providers]]` - API provider configurations
//! - `[[servers]]` - MCP server definitions
//! - `[server]` - REST settings (CORS, docs)

use crate::config::AppConfig;
use crate::config::toml_config;
use crate::logging::ConfigLogger;
use std::error::Error;

/// Generate the client configuration file with a generic provider entry.
///
/// The `provider_type` is an opaque discriminator — core does not interpret it.
/// Each provider adapter (OpenAI, Gemini, Ollama, etc.) is responsible for
/// mapping its own configuration fields.
pub fn generate(
    provider_id: &str,
    provider_type: &str,
    endpoint: &str,
    api_key_env: &str,
    models: &[(String, String)],
) -> Result<(), Box<dyn Error>> {
    let log = ConfigLogger::new("config");

    log.debug(format!(
        "Rendering client config template | provider={}",
        provider_id
    ));

    let mut config = AppConfig::default();

    config.providers.push(crate::config::ProviderConfig {
        id: provider_id.to_string(),
        provider_type: provider_type.to_string(),
        endpoint: endpoint.to_string(),
        api_key: api_key_env.to_string(),
        models: models
            .iter()
            .map(|(name, display)| crate::config::ModelInfo {
                name: name.clone(),
                display_name: display.clone(),
            })
            .collect(),
    });

    toml_config::save_config(&config, None).map_err(|e| {
        log.error(format!("Failed to write config | error={}", e));
        e
    })?;

    log.info(format!(
        "Config generated successfully | path={}",
        toml_config::CONFIG_PATH
    ));
    Ok(())
}

/// Generate the .env file with API keys
pub fn generate_env(api_key_env: &str, api_key: &str) -> Result<(), Box<dyn Error>> {
    let log = ConfigLogger::new("config");
    let env_path = std::path::Path::new(".env");

    let content = if env_path.exists() {
        log.info(format!(
            "Reading existing .env | path={}",
            env_path.display()
        ));
        let existing = std::fs::read_to_string(env_path).map_err(|e| {
            log.error(format!(
                "Failed to read .env | path={} error={}",
                env_path.display(),
                e
            ));
            e
        })?;
        if existing.contains(&format!("{}=", api_key_env)) {
            existing
                .lines()
                .map(|line| {
                    if line.starts_with(&format!("{}=", api_key_env)) {
                        format!("{}={}", api_key_env, api_key)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            let suffix = if existing.ends_with('\n') || existing.is_empty() {
                ""
            } else {
                "\n"
            };
            format!("{}{}{}={}\n", existing, suffix, api_key_env, api_key)
        }
    } else {
        format!("{}={}\n", api_key_env, api_key)
    };

    log.info(format!("Writing .env | path={}", env_path.display()));
    std::fs::write(env_path, content).map_err(|e| {
        log.error(format!(
            "Failed to write .env | path={} error={}",
            env_path.display(),
            e
        ));
        e
    })?;

    Ok(())
}

/// Add a new provider to the config
pub fn add_provider(
    provider_id: &str,
    provider_type: &str,
    endpoint: &str,
    api_key_env: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let log = ConfigLogger::new("config");

    let mut config = load_or_default(&log)?;

    config.providers.push(crate::config::ProviderConfig {
        id: provider_id.to_string(),
        provider_type: provider_type.to_string(),
        endpoint: endpoint.to_string(),
        api_key: api_key_env.unwrap_or("").to_string(),
        models: Vec::new(),
    });

    log.info(format!(
        "Writing config with new provider | provider={}",
        provider_id
    ));
    toml_config::save_config(&config, None).map_err(|e| {
        log.error(format!("Failed to write config | error={}", e));
        e
    })?;

    Ok(())
}

/// Update provider settings in config
pub fn update_provider(
    provider_id: &str,
    new_endpoint: &str,
    new_api_key_env: &str,
) -> Result<(), Box<dyn Error>> {
    let log = ConfigLogger::new("config");

    let mut config = load_or_default(&log)?;

    if let Some(provider) = config.providers.iter_mut().find(|p| p.id == provider_id) {
        provider.endpoint = new_endpoint.to_string();
        provider.api_key = new_api_key_env.to_string();
    } else {
        log.error(format!("Provider not found | provider={}", provider_id));
        return Err(format!("Provider '{}' not found", provider_id).into());
    }

    log.info(format!(
        "Writing updated provider | provider={}",
        provider_id
    ));
    toml_config::save_config(&config, None).map_err(|e| {
        log.error(format!("Failed to write config | error={}", e));
        e
    })?;

    Ok(())
}

/// Add a model to a provider
pub fn add_model_to_provider(
    provider_id: &str,
    model_name: &str,
    display_name: &str,
) -> Result<(), Box<dyn Error>> {
    let log = ConfigLogger::new("config");

    let mut config = load_or_default(&log)?;

    if let Some(provider) = config.providers.iter_mut().find(|p| p.id == provider_id) {
        provider.models.push(crate::config::ModelInfo {
            name: model_name.to_string(),
            display_name: display_name.to_string(),
        });
    } else {
        log.error(format!("Provider not found | provider={}", provider_id));
        return Err(format!("Provider '{}' not found", provider_id).into());
    }

    log.info(format!(
        "Writing config with new model | model={} provider={}",
        model_name, provider_id
    ));
    toml_config::save_config(&config, None).map_err(|e| {
        log.error(format!("Failed to write config | error={}", e));
        e
    })?;

    Ok(())
}

/// Remove a model from a provider
pub fn remove_model_from_provider(
    provider_id: &str,
    model_name: &str,
) -> Result<(), Box<dyn Error>> {
    let log = ConfigLogger::new("config");

    let mut config = load_or_default(&log)?;

    if let Some(provider) = config.providers.iter_mut().find(|p| p.id == provider_id) {
        provider.models.retain(|m| m.name != model_name);
    } else {
        log.error(format!("Provider not found | provider={}", provider_id));
        return Err(format!("Provider '{}' not found", provider_id).into());
    }

    log.info(format!(
        "Writing config after removing model | model={} provider={}",
        model_name, provider_id
    ));
    toml_config::save_config(&config, None).map_err(|e| {
        log.error(format!("Failed to write config | error={}", e));
        e
    })?;

    Ok(())
}

/// Add a server to the config
pub fn add_server(name: &str, command: &str, args: &[String]) -> Result<(), Box<dyn Error>> {
    let log = ConfigLogger::new("config");

    let mut config = load_or_default(&log)?;

    config.servers.push(crate::config::ServerConfig {
        name: name.to_string(),
        transport: crate::config::TransportType::Stdio,
        command: Some(std::path::PathBuf::from(command)),
        args: args.to_vec(),
        env: std::collections::HashMap::new(),
        workdir: None,
        url: None,
        headers: std::collections::HashMap::new(),
        default_timezone: None,
        default_city: None,
    });

    log.info(format!("Writing config with new server | server={}", name));
    toml_config::save_config(&config, None).map_err(|e| {
        log.error(format!("Failed to write config | error={}", e));
        e
    })?;

    Ok(())
}

/// Add an HTTP server to the config
pub fn add_http_server(
    name: &str,
    url: &str,
    headers: &std::collections::HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    let log = ConfigLogger::new("config");

    let mut config = load_or_default(&log)?;

    config.servers.push(crate::config::ServerConfig {
        name: name.to_string(),
        transport: crate::config::TransportType::Http,
        command: None,
        args: Vec::new(),
        env: std::collections::HashMap::new(),
        workdir: None,
        url: Some(url.to_string()),
        headers: headers.clone(),
        default_timezone: None,
        default_city: None,
    });

    log.info(format!(
        "Writing config with new HTTP server | server={}",
        name
    ));
    toml_config::save_config(&config, None).map_err(|e| {
        log.error(format!("Failed to write config | error={}", e));
        e
    })?;

    Ok(())
}

/// Remove a server from the config
pub fn remove_server(server_name: &str) -> Result<(), Box<dyn Error>> {
    let log = ConfigLogger::new("config");

    let mut config = load_or_default(&log)?;

    let before = config.servers.len();
    config.servers.retain(|s| s.name != server_name);

    if config.servers.len() == before {
        log.error(format!("Server not found | server={}", server_name));
        return Err(format!("Server '{}' not found", server_name).into());
    }

    log.info(format!(
        "Writing config after removing server | server={}",
        server_name
    ));
    toml_config::save_config(&config, None).map_err(|e| {
        log.error(format!("Failed to write config | error={}", e));
        e
    })?;

    Ok(())
}

/// Get current CORS origins from config
pub fn get_cors_origins() -> Result<Vec<String>, Box<dyn Error>> {
    let log = ConfigLogger::new("config");

    let config = load_or_default(&log)?;

    Ok(config.server.cors_origins.clone())
}

/// Add a CORS origin to the config
pub fn add_cors_origin(origin: &str) -> Result<(), Box<dyn Error>> {
    let log = ConfigLogger::new("config");

    let mut config = load_or_default(&log)?;

    if config.server.cors_origins.iter().any(|o| o == origin) {
        log.error(format!("Origin already exists | origin={}", origin));
        return Err(format!("Origin '{}' already exists", origin).into());
    }

    config.server.cors_origins.push(origin.to_string());

    log.info(format!(
        "Writing config with new CORS origin | origin={}",
        origin
    ));
    toml_config::save_config(&config, None).map_err(|e| {
        log.error(format!("Failed to write config | error={}", e));
        e
    })?;

    Ok(())
}

/// Remove a CORS origin from the config
pub fn remove_cors_origin(origin: &str) -> Result<(), Box<dyn Error>> {
    let log = ConfigLogger::new("config");

    let mut config = load_or_default(&log)?;

    let before = config.server.cors_origins.len();
    config.server.cors_origins.retain(|o| o != origin);

    if config.server.cors_origins.len() == before {
        log.error(format!("CORS origin not found | origin={}", origin));
        return Err(format!("Origin '{}' not found", origin).into());
    }

    log.info(format!(
        "Writing config after removing CORS origin | origin={}",
        origin
    ));
    toml_config::save_config(&config, None).map_err(|e| {
        log.error(format!("Failed to write config | error={}", e));
        e
    })?;

    Ok(())
}

/// Load config from file, or return a fresh default if not found.
fn load_or_default(log: &crate::logging::ConfigLogger) -> Result<AppConfig, Box<dyn Error>> {
    match toml_config::load_config(None) {
        Ok(config) => Ok(config),
        Err(e) => {
            log.warn(format!(
                "Could not load config, using defaults | error={}",
                e
            ));
            Ok(AppConfig::default())
        }
    }
}
