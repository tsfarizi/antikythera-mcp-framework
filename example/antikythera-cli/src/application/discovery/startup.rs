//! Startup server discovery integration.

use super::types::{DiscoveredServer, DiscoverySummary, LoadStatus};
use super::{DEFAULT_SERVERS_FOLDER, load_all, scan_folder};
use antikythera_core::logging::DiscoveryLogger;
use std::path::Path;

/// Result of the startup discovery process.
#[derive(Debug, Clone)]
pub struct StartupDiscoveryResult {
    /// All discovered servers
    pub servers: Vec<DiscoveredServer>,
    /// Summary statistics
    pub summary: DiscoverySummary,
    /// Whether the servers folder exists
    pub folder_exists: bool,
}

impl StartupDiscoveryResult {
    pub fn has_loaded_servers(&self) -> bool {
        self.summary.loaded > 0
    }

    pub fn loaded_servers(&self) -> Vec<&DiscoveredServer> {
        self.servers.iter().filter(|s| s.is_loaded()).collect()
    }

    pub fn failed_servers(&self) -> Vec<&DiscoveredServer> {
        self.servers
            .iter()
            .filter(|s| matches!(s.load_status, LoadStatus::Failed(_)))
            .collect()
    }
}

/// Run server discovery at startup and log results.
pub async fn run_startup_discovery(servers_folder: Option<&Path>) -> StartupDiscoveryResult {
    let log = DiscoveryLogger::new("discovery");
    let folder = servers_folder
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new(DEFAULT_SERVERS_FOLDER).to_path_buf());

    log.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    log.info("🔍 MCP Server Discovery");
    log.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    log.info(format!(
        "Scanning servers folder | path={}",
        folder.display()
    ));

    if !folder.exists() {
        log.warn(format!(
            "Servers folder not found - skipping discovery | path={}",
            folder.display()
        ));
        return StartupDiscoveryResult {
            servers: Vec::new(),
            summary: DiscoverySummary::default(),
            folder_exists: false,
        };
    }

    let mut servers = match scan_folder(&folder) {
        Ok(s) => s,
        Err(e) => {
            log.error(format!("Failed to scan servers folder | error={}", e));
            return StartupDiscoveryResult {
                servers: Vec::new(),
                summary: DiscoverySummary::default(),
                folder_exists: true,
            };
        }
    };

    if servers.is_empty() {
        log.info("No server binaries found in folder");
        return StartupDiscoveryResult {
            servers: Vec::new(),
            summary: DiscoverySummary::default(),
            folder_exists: true,
        };
    }

    log.info(format!("Found server binaries | count={}", servers.len()));

    for server in &servers {
        log.info(format!(
            "📦 Discovered server binary | name={} path={}",
            server.name,
            server.binary_path.display()
        ));
    }

    log.info("⏳ Loading servers and fetching tools via MCP...");

    let summary = load_all(&mut servers).await;

    log.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    log.info("📋 Discovery Results");
    log.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for server in &servers {
        match &server.load_status {
            LoadStatus::Success => {
                log.info(format!(
                    "✅ Server loaded successfully | name={} tools={}",
                    server.name,
                    server.tools.len()
                ));
                for (tool_name, description) in &server.tools {
                    let desc_preview: String = description.chars().take(50).collect();
                    log.info(format!(
                        "   🔧 Tool available | server={} tool={} desc={}",
                        server.name, tool_name, desc_preview
                    ));
                }
            }
            LoadStatus::NoTools => {
                log.warn(format!(
                    "⚠️  Server loaded but has no tools | name={}",
                    server.name
                ));
            }
            LoadStatus::Failed(err) => {
                log.error(format!(
                    "❌ Failed to load server | name={} error={}",
                    server.name, err
                ));
            }
            LoadStatus::Pending => {
                log.warn(format!(
                    "⏳ Server not loaded (pending) | name={}",
                    server.name
                ));
            }
        }
    }

    log.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    log.info(format!(
        "📊 Discovery Summary | total={} loaded={} failed={} no_tools={} total_tools={}",
        summary.total_found, summary.loaded, summary.failed, summary.no_tools, summary.total_tools
    ));
    log.info("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    StartupDiscoveryResult {
        servers,
        summary,
        folder_exists: true,
    }
}

/// Print discovery results to stdout.
pub fn print_discovery_summary(result: &StartupDiscoveryResult) {
    if !result.folder_exists {
        antikythera_log::cli_print!("⚠️  Servers folder not found - no auto-discovery performed");
        return;
    }

    if result.servers.is_empty() {
        antikythera_log::cli_print!("📂 No server binaries found in servers folder");
        return;
    }

    antikythera_log::cli_print!();
    antikythera_log::cli_print!("🔍 MCP Server Discovery Results:");
    antikythera_log::cli_print!("─────────────────────────────────");

    for server in &result.servers {
        match &server.load_status {
            LoadStatus::Success => {
                antikythera_log::cli_print!("  ✅ {} ({} tools)", server.name, server.tools.len());
                for (tool_name, _) in &server.tools {
                    antikythera_log::cli_print!("     └─ {}", tool_name);
                }
            }
            LoadStatus::NoTools => {
                antikythera_log::cli_print!("  ⚠️  {} (no tools)", server.name);
            }
            LoadStatus::Failed(e) => {
                antikythera_log::cli_print!("  ❌ {} - Error: {}", server.name, e);
            }
            LoadStatus::Pending => {}
        }
    }

    antikythera_log::cli_print!("─────────────────────────────────");
    antikythera_log::cli_print!(
        "📊 Total: {} servers | ✅ {} loaded | ❌ {} failed | 🔧 {} tools",
        result.summary.total_found,
        result.summary.loaded,
        result.summary.failed,
        result.summary.total_tools
    );
    antikythera_log::cli_print!();
}
