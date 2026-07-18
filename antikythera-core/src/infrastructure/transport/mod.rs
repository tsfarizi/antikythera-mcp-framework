//! Concrete transport adapter implementations.
//!
//! This module contains the infrastructure-level implementations of MCP
//! transport adapters (HTTP, STDIO, Builtin) that implement the port
//! trait defined in `application::tooling::transport::McpTransport`.

mod builtin;
mod config;
mod http;
#[cfg(feature = "native-transport")]
pub mod stdio;

#[cfg(feature = "native-transport")]
mod tool_catalogue;
mod factory;

pub use builtin::{BuiltinToolFn, BuiltinTransport, validate_arguments};
pub use config::{HttpTransportConfig, TransportMode};
pub use http::HttpTransport;
pub use factory::TransportFactory;
#[cfg(feature = "native-transport")]
pub use tool_catalogue::spawn_and_list_tools;
