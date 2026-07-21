//! Concrete transport adapter implementations (CLI-owned).

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
pub use factory::CliTransportFactory;
#[cfg(feature = "native-transport")]
pub use tool_catalogue::spawn_and_list_tools;
