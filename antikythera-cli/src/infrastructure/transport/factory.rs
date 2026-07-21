use antikythera_core::application::tooling::error::ToolInvokeError;
use antikythera_core::application::tooling::{ServerInstance, TransportFactory};
use antikythera_core::application::tooling::transport::McpTransport;
#[cfg(feature = "native-transport")]
use super::stdio::McpProcess;
use super::{HttpTransport, HttpTransportConfig, TransportMode};
use antikythera_core::config::{ServerConfig, TransportType};
use async_trait::async_trait;
use std::sync::Arc;

/// CLI implementation of `TransportFactory`.
///
/// Creates concrete transport instances from `ServerConfig` and wraps them
/// in `ServerInstance`.
pub struct CliTransportFactory;

impl Default for CliTransportFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl CliTransportFactory {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TransportFactory for CliTransportFactory {
    async fn create(&self, config: &ServerConfig) -> Result<ServerInstance, ToolInvokeError> {
        match config.transport {
            TransportType::Stdio => {
                #[cfg(feature = "native-transport")]
                {
                    let process = Arc::new(McpProcess::new(config.clone()));
                    process.ensure_running().await?;
                    Ok(ServerInstance::new(process))
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
                };
                let transport = Arc::new(HttpTransport::new(transport_config));
                transport.connect().await?;
                Ok(ServerInstance::new(transport))
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
