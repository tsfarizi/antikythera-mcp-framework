//! Tool routing: resolve a tool call to one of three destinations
//! (`local` / `remote` / `mcp`) and execute it behind the permission gate.
//!
//! - `local` — server tools: handler functions registered by the SDK user.
//! - `remote` — client tools: `tool-execution-request` on the SSE control
//!   channel + POST-back, fail-closed on missing client or TTL expiry.
//! - `mcp` — third-party MCP servers via `antikythera-tooling` transports,
//!   always server-side.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use antikythera_config::{ServerConfig, TransportType};
use serde_json::Value;

use crate::config::GatePolicy;
use crate::control::{ControlChannel, PendingKind};
use crate::mcp::{SharedMcpTransport, mcp_result_to_execution, server_info_to_definition};
use crate::registry::{Destination, ToolOwner, UnionRegistry};
use crate::wire::{EventEnvelope, ToolCallEvent, ToolDefinition, ToolExecutionResult};

/// Handler for a server-side (local) tool: arguments object in, result out.
/// A tool failure is NOT an error: return `Ok(Value)` with an error-shaped
/// object, or surface an execution-level failure with `Err(message)`.
pub type LocalToolHandler = dyn Fn(Value) -> Result<Value, String> + Send + Sync;

/// The routing engine shared by host functions, the tool loop, and the HTTP
/// `POST /tools/execute` endpoint.
pub struct ToolRouter {
    registry: Mutex<UnionRegistry>,
    local_handlers: Mutex<HashMap<String, Arc<LocalToolHandler>>>,
    mcp_servers: Mutex<HashMap<String, SharedMcpTransport>>,
    tool_server: Mutex<HashMap<String, String>>,
    control: Arc<ControlChannel>,
    policy: Arc<GatePolicy>,
    runtime: tokio::runtime::Handle,
    client_id: String,
    pending_ttl: Duration,
}

impl ToolRouter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        control: Arc<ControlChannel>,
        policy: Arc<GatePolicy>,
        runtime: tokio::runtime::Handle,
        client_id: String,
        pending_ttl: Duration,
    ) -> Self {
        Self {
            registry: Mutex::new(UnionRegistry::new()),
            local_handlers: Mutex::new(HashMap::new()),
            mcp_servers: Mutex::new(HashMap::new()),
            tool_server: Mutex::new(HashMap::new()),
            control,
            policy,
            runtime,
            client_id,
            pending_ttl,
        }
    }

    /// Register a server-side tool with its local handler.
    pub fn register_local_tool(
        &self,
        definition: ToolDefinition,
        handler: Arc<LocalToolHandler>,
    ) -> Result<(), String> {
        let name = definition.name.clone();
        self.registry
            .lock()
            .expect("tool registry lock poisoned")
            .register(ToolOwner::Server, definition)?;
        self.local_handlers
            .lock()
            .expect("tool handlers lock poisoned")
            .insert(name, handler);
        Ok(())
    }

    /// Register client-side (remote) tool definitions.
    pub fn register_client_tools(&self, definitions: Vec<ToolDefinition>) -> Result<(), String> {
        let mut registry = self.registry.lock().expect("tool registry lock poisoned");
        for definition in definitions {
            registry.register(ToolOwner::Client, definition)?;
        }
        Ok(())
    }

    /// Connect an MCP server and register its tools as `mcp`-owned. Blocking:
    /// must be called from a non-async context (the HTTP server drives async
    /// work on its own threads).
    pub fn connect_mcp_server(&self, config: ServerConfig) -> Result<(), String> {
        let transport: SharedMcpTransport = match config.transport {
            TransportType::Stdio => Arc::new(crate::mcp::StdioMcpTransport::new(config.clone())),
            TransportType::Http => Arc::new(crate::mcp::HttpMcpTransport::new(config.clone())),
            TransportType::Builtin => {
                return Err(
                    "mcp: builtin transport is not supported by the server runtime".to_string(),
                );
            }
        };
        let server_name = transport.server_name().to_string();
        self.runtime.block_on(async {
            transport
                .connect()
                .await
                .map_err(|e| format!("mcp: connect '{server_name}' failed: {e}"))?;
            let tools = transport.list_tools().await;
            let mut registry = self.registry.lock().expect("tool registry lock poisoned");
            let mut tool_server = self.tool_server.lock().expect("tool server lock poisoned");
            for info in tools {
                let name = info.name.clone();
                let definition = server_info_to_definition(&info);
                registry.register(ToolOwner::Mcp, definition)?;
                tool_server.insert(name, server_name.clone());
            }
            drop(tool_server);
            drop(registry);
            self.mcp_servers
                .lock()
                .expect("mcp servers lock poisoned")
                .insert(server_name, transport);
            Ok(())
        })
    }

    pub fn owner_of(&self, tool: &str) -> Option<ToolOwner> {
        self.registry
            .lock()
            .expect("tool registry lock poisoned")
            .owner_of(tool)
    }

    /// Resolve the routing destination for a tool call. Unknown tools are
    /// denied (no owner means no destination).
    pub fn resolve_destination(&self, tool: &str) -> Result<Destination, String> {
        let registry = self.registry.lock().expect("tool registry lock poisoned");
        match registry.owner_of(tool) {
            Some(owner) => Ok(Destination::from(owner)),
            None => Err(format!("permission: tool '{tool}' not in allowlist")),
        }
    }

    /// The union of all registered definitions (for the single
    /// `register-tools` push to the runner).
    pub fn union_definitions(&self) -> Vec<ToolDefinition> {
        self.registry
            .lock()
            .expect("tool registry lock poisoned")
            .definitions()
    }

    /// Peer-facing definitions (`GET /tools`): server- and mcp-owned tools.
    pub fn peer_definitions(&self) -> Vec<ToolDefinition> {
        let registry = self.registry.lock().expect("tool registry lock poisoned");
        let mut defs = registry.definitions_for(ToolOwner::Server);
        defs.extend(registry.definitions_for(ToolOwner::Mcp));
        defs
    }

    /// Execute a tool call with full routing (host-imports `emit-tool-call`
    /// and the tool loop).
    pub async fn execute(&self, event: &ToolCallEvent) -> Result<ToolExecutionResult, String> {
        let destination = self.resolve_destination(&event.tool_name)?;
        self.execute_for(destination, event).await
    }

    /// Execute a tool call that MUST be server- or mcp-owned
    /// (`POST /tools/execute`). Client-owned tools are denied here.
    pub async fn execute_server_owned(
        &self,
        event: &ToolCallEvent,
    ) -> Result<ToolExecutionResult, String> {
        match self.resolve_destination(&event.tool_name)? {
            Destination::Remote => Err(format!(
                "permission: tool '{}' is owned by the client; server cannot execute it",
                event.tool_name
            )),
            destination => self.execute_for(destination, event).await,
        }
    }

    async fn execute_for(
        &self,
        destination: Destination,
        event: &ToolCallEvent,
    ) -> Result<ToolExecutionResult, String> {
        self.policy.check_tool(destination, &event.tool_name)?;
        match destination {
            Destination::Local => self.execute_local(event),
            Destination::Remote => self.execute_remote(event).await,
            Destination::Mcp => self.execute_mcp(event).await,
        }
    }

    fn execute_local(&self, event: &ToolCallEvent) -> Result<ToolExecutionResult, String> {
        let handler = self
            .local_handlers
            .lock()
            .expect("tool handlers lock poisoned")
            .get(&event.tool_name)
            .cloned()
            .ok_or_else(|| format!("tool '{}' has no local handler", event.tool_name))?;
        let arguments: Value = serde_json::from_str(&event.arguments_json).map_err(|e| {
            format!(
                "tool '{}': cannot parse arguments-json: {e}",
                event.tool_name
            )
        })?;
        match handler(arguments) {
            Ok(output) => Ok(ToolExecutionResult {
                tool_name: event.tool_name.clone(),
                success: true,
                output_json: output.to_string(),
                error_message: None,
                step_id: event.step_id,
            }),
            Err(message) => Ok(ToolExecutionResult {
                tool_name: event.tool_name.clone(),
                success: false,
                output_json: "{}".to_string(),
                error_message: Some(message),
                step_id: event.step_id,
            }),
        }
    }

    async fn execute_remote(&self, event: &ToolCallEvent) -> Result<ToolExecutionResult, String> {
        let correlation_id = uuid::Uuid::new_v4().to_string();
        let mut rx = self.control.register_pending(
            correlation_id.clone(),
            PendingKind::Tool,
            self.pending_ttl,
        );
        if !self.control.is_client_connected(&self.client_id) {
            self.control.cancel_pending(&correlation_id);
            return Err(format!(
                "permission: tool '{}' requires a connected client",
                event.tool_name
            ));
        }
        let envelope = EventEnvelope {
            event_type: "tool-execution-request".to_string(),
            correlation_id: Some(correlation_id.clone()),
            session_id: event.session_id.clone(),
            client_id: self.client_id.clone(),
            payload: serde_json::to_value(event).unwrap_or(Value::Null),
        };
        self.control.push(&self.client_id, &envelope);
        match tokio::time::timeout(self.pending_ttl, &mut rx).await {
            Ok(Ok(body)) => {
                if !body.ok {
                    return Err(normalize_denial(body.error.unwrap_or_else(|| {
                        format!("permission: tool '{}' rejected by client", event.tool_name)
                    })));
                }
                let result: ToolExecutionResult =
                    serde_json::from_value(body.payload).map_err(|e| {
                        format!(
                            "tool '{}': client returned invalid tool-execution-result: {e}",
                            event.tool_name
                        )
                    })?;
                Ok(result)
            }
            Ok(Err(_)) => {
                self.control.cancel_pending(&correlation_id);
                Err(format!(
                    "permission: tool '{}' timed out waiting for client response",
                    event.tool_name
                ))
            }
            Err(_) => {
                self.control.cancel_pending(&correlation_id);
                Err(format!(
                    "permission: tool '{}' timed out waiting for client response",
                    event.tool_name
                ))
            }
        }
    }

    async fn execute_mcp(&self, event: &ToolCallEvent) -> Result<ToolExecutionResult, String> {
        let server = self
            .tool_server
            .lock()
            .expect("tool server lock poisoned")
            .get(&event.tool_name)
            .cloned()
            .ok_or_else(|| format!("tool '{}' has no MCP server mapping", event.tool_name))?;
        let transport = self
            .mcp_servers
            .lock()
            .expect("mcp servers lock poisoned")
            .get(&server)
            .cloned()
            .ok_or_else(|| format!("mcp server '{server}' is not connected"))?;
        let arguments: Value = serde_json::from_str(&event.arguments_json).map_err(|e| {
            format!(
                "tool '{}': cannot parse arguments-json: {e}",
                event.tool_name
            )
        })?;
        let result = transport
            .call_tool(&event.tool_name, arguments)
            .await
            .map_err(|e| format!("mcp: tool '{}' failed on '{server}': {e}", event.tool_name))?;
        Ok(mcp_result_to_execution(
            &event.tool_name,
            result,
            event.step_id,
        ))
    }
}

/// A denial message must start with `permission:`; normalize client-provided
/// errors so the invariant holds.
fn normalize_denial(message: String) -> String {
    if message.starts_with("permission:") {
        message
    } else {
        format!("permission: {message}")
    }
}

impl ToolRouter {
    #[cfg(test)]
    fn control_pending_len_for_test(&self) -> usize {
        self.control.pending_len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GatePolicy;
    use crate::control::ControlChannel;
    use crate::wire::PostbackBody;

    fn test_router(policy: GatePolicy) -> (Arc<ToolRouter>, Arc<ControlChannel>) {
        let control = Arc::new(ControlChannel::new());
        let router = Arc::new(ToolRouter::new(
            control.clone(),
            Arc::new(policy),
            tokio::runtime::Handle::current(),
            "client-a".to_string(),
            Duration::from_secs(5),
        ));
        (router, control)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn routing_decision_resolves_owner_to_destination() {
        let (router, _) = test_router(GatePolicy::deny_all());
        router
            .register_local_tool(
                ToolDefinition::simple("local_tool", "server tool"),
                Arc::new(Ok),
            )
            .unwrap();
        router
            .register_client_tools(vec![ToolDefinition::simple("client_tool", "client tool")])
            .unwrap();

        assert_eq!(
            router.resolve_destination("local_tool").unwrap(),
            Destination::Local
        );
        assert_eq!(
            router.resolve_destination("client_tool").unwrap(),
            Destination::Remote
        );
        assert!(router.resolve_destination("unknown_tool").is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn gate_denial_blocks_execution_before_dispatch() {
        let (router, _) = test_router(GatePolicy::deny_all());
        router
            .register_local_tool(
                ToolDefinition::simple("echo", "server tool"),
                Arc::new(|_| Ok(serde_json::json!({"echoed": true}))),
            )
            .unwrap();
        let event = ToolCallEvent {
            tool_name: "echo".to_string(),
            arguments_json: "{}".to_string(),
            session_id: None,
            step_id: 1,
        };
        // The tool is registered but the default-deny policy blocks it.
        let err = router.execute(&event).await.unwrap_err();
        assert!(err.starts_with("permission: "), "got: {err}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn local_execution_runs_registered_handler() {
        let mut policy = GatePolicy::deny_all();
        policy.allow_tool(Destination::Local, "echo");
        let (router, _) = test_router(policy);
        router
            .register_local_tool(
                ToolDefinition::simple("echo", "server tool"),
                Arc::new(|args| Ok(serde_json::json!({"echoed": args}))),
            )
            .unwrap();
        let event = ToolCallEvent {
            tool_name: "echo".to_string(),
            arguments_json: "{\"x\": 1}".to_string(),
            session_id: None,
            step_id: 2,
        };
        let result = router.execute(&event).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output_json, "{\"echoed\":{\"x\":1}}");
        assert_eq!(result.step_id, 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn local_handler_failure_is_a_result_not_an_error() {
        let mut policy = GatePolicy::deny_all();
        policy.allow_tool(Destination::Local, "boom");
        let (router, _) = test_router(policy);
        router
            .register_local_tool(
                ToolDefinition::simple("boom", "fails"),
                Arc::new(|_| Err("handler exploded".to_string())),
            )
            .unwrap();
        let event = ToolCallEvent {
            tool_name: "boom".to_string(),
            arguments_json: "{}".to_string(),
            session_id: None,
            step_id: 1,
        };
        let result = router.execute(&event).await.unwrap();
        assert!(!result.success);
        assert_eq!(result.error_message.as_deref(), Some("handler exploded"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn remote_execution_roundtrips_through_postback() {
        let mut policy = GatePolicy::deny_all();
        policy.allow_tool(Destination::Remote, "client_tool");
        let (router, control) = test_router(policy);
        router
            .register_client_tools(vec![ToolDefinition::simple("client_tool", "client tool")])
            .unwrap();

        let mut rx = control.register_client("client-a");
        let client = control.clone();
        let responder = tokio::spawn(async move {
            // Read the SSE envelope, answer the tool-execution-request.
            while let Ok(json) = rx.recv().await {
                let value: Value = serde_json::from_str(&json).unwrap();
                if value["type"] == "tool-execution-request" {
                    let correlation_id = value["correlation_id"].as_str().unwrap().to_string();
                    client.complete_postback(PostbackBody {
                        correlation_id,
                        ok: true,
                        payload: serde_json::json!({
                            "tool-name": "client_tool",
                            "success": true,
                            "output-json": "{\"done\":true}",
                            "error-message": null,
                            "step-id": 1,
                        }),
                        error: None,
                    });
                    break;
                }
            }
        });

        let event = ToolCallEvent {
            tool_name: "client_tool".to_string(),
            arguments_json: "{}".to_string(),
            session_id: None,
            step_id: 1,
        };
        let result = router.execute(&event).await.unwrap();
        responder.await.unwrap();
        assert!(result.success);
        assert_eq!(result.output_json, "{\"done\":true}");
        assert_eq!(result.tool_name, "client_tool");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn remote_execution_fails_closed_without_client() {
        let mut policy = GatePolicy::deny_all();
        policy.allow_tool(Destination::Remote, "client_tool");
        let (router, _) = test_router(policy);
        router
            .register_client_tools(vec![ToolDefinition::simple("client_tool", "client tool")])
            .unwrap();
        // No client registered.
        let event = ToolCallEvent {
            tool_name: "client_tool".to_string(),
            arguments_json: "{}".to_string(),
            session_id: None,
            step_id: 1,
        };
        let err = router.execute(&event).await.unwrap_err();
        assert!(err.starts_with("permission: "), "got: {err}");
        assert_eq!(router.control_pending_len_for_test(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn server_owned_execution_denies_client_tools() {
        let mut policy = GatePolicy::deny_all();
        policy.allow_tool(Destination::Remote, "client_tool");
        let (router, _) = test_router(policy);
        router
            .register_client_tools(vec![ToolDefinition::simple("client_tool", "client tool")])
            .unwrap();
        let event = ToolCallEvent {
            tool_name: "client_tool".to_string(),
            arguments_json: "{}".to_string(),
            session_id: None,
            step_id: 1,
        };
        let err = router.execute_server_owned(&event).await.unwrap_err();
        assert!(err.starts_with("permission: "), "got: {err}");
        assert!(err.contains("owned by the client"));
    }
}
