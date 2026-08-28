use super::McpClient;
use crate::application::config::ToolConfig;
use crate::application::tooling::ToolServerInterface;
use crate::infrastructure::model::ModelProvider;
use std::sync::Arc;

impl<P: ModelProvider> McpClient<P> {
    /// Return the list of registered tool configurations.
    pub fn tools(&self) -> &[ToolConfig] {
        &self.config.tools
    }

    /// Return a clone of the active [`ToolServerInterface`] arc (the `ServerManager`).
    pub fn server_bridge(&self) -> Arc<dyn ToolServerInterface> {
        self.server_bridge.clone()
    }
}
