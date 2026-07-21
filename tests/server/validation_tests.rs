// Server validation tests - validating server configuration
//
// Tests that verify configuration references are valid.
// These tests gracefully skip if config files don't exist.

use antikythera_cli::application::discovery::loader::create_server_config;
use antikythera_cli::application::discovery::scanner::{extract_server_name, is_executable};
use antikythera_cli::application::discovery::{
    DEFAULT_SERVERS_FOLDER, DiscoveredServer, DiscoveryError, DiscoverySummary, LoadStatus,
    StartupDiscoveryResult, load_server, scan_folder,
};
use antikythera_core::config::AppConfig;
use antikythera_core::config::server::{RawServer, ServerConfig};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

// Split into 11 parts for consistent test organization.
include!("validation_tests/empty_placeholder.rs");
include!("validation_tests/empty_placeholder.rs");
include!("validation_tests/server_commands_valid.rs");
include!("validation_tests/empty_placeholder.rs");
include!("validation_tests/tool_server_refs.rs");
include!("validation_tests/server_config_types.rs");
include!("validation_tests/discovered_server_types.rs");
include!("validation_tests/scanner_folder.rs");
include!("validation_tests/config_loader.rs");
include!("validation_tests/startup_discovery.rs");
include!("validation_tests/default_folder_reexports.rs");
