use super::error::ToolInvokeError;
use super::manager::ServerInstance;
#[cfg(feature = "native-transport")]
use super::process::McpProcess;
use super::transport::{HttpTransport, HttpTransportConfig, McpTransport, TransportMode};
use crate::config::{ServerConfig, TransportType};
use std::sync::Arc;

/// Creates `ServerInstance` variants from a `ServerConfig`.
///
/// Extracted from `ServerManager::ensure_instance` so that transport
/// construction logic lives in a single, testable place.
pub struct TransportFactory;

impl Default for TransportFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportFactory {
    pub fn new() -> Self {
        Self
    }

    pub(crate) async fn create(&self, config: &ServerConfig) -> Result<ServerInstance, ToolInvokeError> {
        match config.transport {
            TransportType::Stdio => {
                #[cfg(feature = "native-transport")]
                {
                    let process = Arc::new(McpProcess::new(config.clone()));
                    process.ensure_running().await?;
                    Ok(ServerInstance::Stdio(process))
                }
                #[cfg(not(feature = "native-transport"))]
                {
                    Err(ToolInvokeError::Transport {
                        server: config.name.clone(),
                        message: "STDIO transport requires the native-transport feature"
                            .to_string(),
                    })
                }
            }
            TransportType::Http => {
                let url = config
                    .url
                    .clone()
                    .ok_or_else(|| ToolInvokeError::NotConfigured {
                        server: format!("{}: missing URL for HTTP transport", config.name),
                    })?;
                let transport_config = HttpTransportConfig {
                    name: config.name.clone(),
                    url,
                    headers: config.headers.clone(),
                    mode: TransportMode::Auto,
                    required_capabilities: Vec::new(),
                };
                let transport = Arc::new(HttpTransport::new(transport_config));
                transport.connect().await?;
                Ok(ServerInstance::Http(transport))
            }
            TransportType::Builtin => Err(ToolInvokeError::NotConfigured {
                server: format!(
                    "{}: builtin transport must be pre-registered via register_builtin_transport()",
                    config.name
                ),
            }),
        }
    }
}
