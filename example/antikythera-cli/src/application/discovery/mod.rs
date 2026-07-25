//! MCP Server Discovery Module

#[cfg(feature = "native-transport")]
pub mod loader;
pub mod scanner;
#[cfg(feature = "native-transport")]
pub mod startup;
pub mod types;

#[cfg(feature = "native-transport")]
pub use loader::{load_all, load_server, scan_and_load};
pub use scanner::scan_folder;
#[cfg(feature = "native-transport")]
pub use startup::{StartupDiscoveryResult, print_discovery_summary, run_startup_discovery};
pub use types::{DiscoveredServer, DiscoveryError, DiscoverySummary, LoadStatus};

/// Default folder name for MCP server binaries.
pub const DEFAULT_SERVERS_FOLDER: &str = "servers";
