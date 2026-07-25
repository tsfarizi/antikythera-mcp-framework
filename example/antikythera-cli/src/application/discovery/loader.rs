//! Server Loader

use super::types::{DiscoveredServer, DiscoverySummary, LoadStatus};
use crate::infrastructure::transport::spawn_and_list_tools;
use antikythera_core::application::config::ServerConfig;
use antikythera_core::logging::DiscoveryLogger;
use std::collections::HashMap;
use std::path::Path;

/// Load all discovered servers and fetch their tools.
pub async fn load_all(servers: &mut [DiscoveredServer]) -> DiscoverySummary {
    let log = DiscoveryLogger::new("discovery");
    log.info(format!(
        "Loading discovered servers | count={}",
        servers.len()
    ));

    for server in servers.iter_mut() {
        load_server(server).await;
    }

    let summary = DiscoverySummary::from_servers(servers);

    log.info(format!(
        "Server loading complete | loaded={} failed={} total_tools={}",
        summary.loaded, summary.failed, summary.total_tools
    ));

    summary
}

/// Load a single server and fetch its tools.
pub async fn load_server(server: &mut DiscoveredServer) {
    let log = DiscoveryLogger::new("discovery");
    log.debug(format!(
        "Loading server | name={} path={}",
        server.name,
        server.binary_path.display()
    ));

    let config = create_server_config(&server.name, &server.binary_path);

    match spawn_and_list_tools(&config).await {
        Ok(tools) => {
            if tools.is_empty() {
                log.info(format!(
                    "Server loaded but has no tools | name={}",
                    server.name
                ));
                server.load_status = LoadStatus::NoTools;
            } else {
                log.info(format!(
                    "Server loaded successfully | name={} tool_count={}",
                    server.name,
                    tools.len()
                ));

                for (name, desc) in &tools {
                    log.debug(format!(
                        "Discovered tool | server={} tool={} description={}",
                        server.name, name, desc
                    ));
                }

                server.tools = tools;
                server.load_status = LoadStatus::Success;
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            log.error(format!(
                "Failed to load server | name={} error={}",
                server.name, error_msg
            ));
            server.load_status = LoadStatus::Failed(error_msg);
        }
    }
}

use antikythera_core::application::config::TransportType;

/// Create a `ServerConfig` from a binary path.
pub fn create_server_config(name: &str, binary_path: &Path) -> ServerConfig {
    ServerConfig {
        name: name.to_string(),
        transport: TransportType::Stdio,
        command: Some(binary_path.to_path_buf()),
        args: Vec::new(),
        env: HashMap::new(),
        workdir: None,
        url: None,
        headers: HashMap::new(),
        default_timezone: None,
        default_city: None,
    }
}

/// Scan and load all servers from a folder in one operation.
pub async fn scan_and_load(
    folder_path: impl AsRef<std::path::Path>,
) -> Result<(Vec<DiscoveredServer>, DiscoverySummary), super::types::DiscoveryError> {
    use super::scanner;

    let mut servers = scanner::scan_folder(folder_path)?;

    if servers.is_empty() {
        DiscoveryLogger::new("discovery").warn("No servers found in folder");
        return Ok((servers, DiscoverySummary::default()));
    }

    let summary = load_all(&mut servers).await;
    Ok((servers, summary))
}
