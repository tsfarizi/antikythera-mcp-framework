//! Server runtime configuration and the default-deny permission gate policy.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use antikythera_config::ServerConfig;

use crate::registry::Destination;

/// A runtime-hook pipeline point whose decisions may be delegated to the
/// peer over the SSE control channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HookName {
    PrepareTurn,
    DecideAction,
    HandleToolResult,
}

impl HookName {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookName::PrepareTurn => "prepare-turn",
            HookName::DecideAction => "decide-action",
            HookName::HandleToolResult => "handle-tool-result",
        }
    }
}

/// Default-deny permission gate policy.
///
/// Every denial surfaces as an error whose message starts with `permission:`
/// (repo invariant). Tools are denied unless allowlisted for their
/// destination; hooks are denied unless allowlisted; LLM calls are allowed
/// with an optional per-session quota.
#[derive(Debug, Clone, Default)]
pub struct GatePolicy {
    /// Allowlist of local (server-side) tools. `None` = deny all local tools.
    pub local_tools: Option<HashSet<String>>,
    /// Allowlist of remote (client-side) tools. `None` = deny all remote tools.
    pub remote_tools: Option<HashSet<String>>,
    /// Allowlist of MCP tools. `None` = deny all MCP tools.
    pub mcp_tools: Option<HashSet<String>>,
    /// Hook points allowed to invoke the peer. Empty = deny all hooks.
    pub allowed_hooks: HashSet<HookName>,
    /// Per-session LLM call quota. `None` = unlimited.
    pub llm_quota: Option<u32>,
}

impl GatePolicy {
    /// A policy that denies everything (the default posture).
    pub fn deny_all() -> Self {
        Self::default()
    }

    /// Allow a tool for a destination.
    pub fn allow_tool(&mut self, destination: Destination, tool: impl Into<String>) {
        let set = match destination {
            Destination::Local => &mut self.local_tools,
            Destination::Remote => &mut self.remote_tools,
            Destination::Mcp => &mut self.mcp_tools,
        };
        set.get_or_insert_with(HashSet::new).insert(tool.into());
    }

    /// Allow a hook point to delegate to the peer.
    pub fn allow_hook(&mut self, hook: HookName) {
        self.allowed_hooks.insert(hook);
    }

    /// Set the per-session LLM quota.
    pub fn set_llm_quota(&mut self, quota: u32) {
        self.llm_quota = Some(quota);
    }

    /// Check that `tool` may execute at `destination`.
    pub fn check_tool(&self, destination: Destination, tool: &str) -> Result<(), String> {
        let allowed = match destination {
            Destination::Local => self.local_tools.as_ref(),
            Destination::Remote => self.remote_tools.as_ref(),
            Destination::Mcp => self.mcp_tools.as_ref(),
        };
        match allowed {
            Some(set) if set.contains(tool) => Ok(()),
            _ => Err(format!("permission: tool '{tool}' not in allowlist")),
        }
    }

    /// Check that a hook point may invoke the peer.
    pub fn check_hook(&self, hook: HookName) -> Result<(), String> {
        if self.allowed_hooks.contains(&hook) {
            Ok(())
        } else {
            Err(format!("permission: hook '{}' not allowed", hook.as_str()))
        }
    }

    /// Check that a session may issue another LLM call under the quota.
    /// `used` is the number of calls already consumed by the session.
    pub fn check_llm(&self, used: u32) -> Result<(), String> {
        match self.llm_quota {
            Some(quota) if used >= quota => Err("permission: llm quota exceeded".to_string()),
            _ => Ok(()),
        }
    }
}

/// LLM provider specification resolved at server construction.
#[derive(Debug, Clone)]
pub enum LlmProviderSpec {
    /// Deterministic provider used by tests and the smoke binary; returns a
    /// fixed framework-generic response content.
    Stub { response: String },
    /// Local-dev Ollama via the `antikythera-provider-ollama` client.
    Ollama { endpoint: String, model: String },
    /// OpenAI-compatible endpoint via the `antikythera-provider-openai` client.
    OpenAi {
        endpoint: String,
        api_key: String,
        model: String,
    },
}

/// A server-side tool registered from the CLI (`--server-tool <name>:<json>`).
/// Registration is a grant: the tool is both registered in the union registry
/// (so `GET /tools` lists it and routing resolves it to `local`) and allowlisted
/// for the local destination, so `--allow-tool` is NOT required.
#[derive(Debug, Clone)]
pub struct ServerToolSpec {
    pub name: String,
    pub response_json: serde_json::Value,
}

impl ServerToolSpec {
    /// Parse `<name>:<response-json>`; the name is everything before the FIRST
    /// colon, the remainder must parse as JSON (explicit error otherwise).
    pub fn parse(spec: &str) -> Result<Self, String> {
        let colon = spec.find(':').ok_or_else(|| {
            format!("invalid --server-tool '{spec}': expected <name>:<response-json>")
        })?;
        let name = spec[..colon].trim().to_string();
        if name.is_empty() {
            return Err(format!(
                "invalid --server-tool '{spec}': tool name must not be empty"
            ));
        }
        let response_json = serde_json::from_str(&spec[colon + 1..])
            .map_err(|e| format!("invalid --server-tool response-json for tool '{name}': {e}"))?;
        Ok(Self {
            name,
            response_json,
        })
    }
}

/// Server runtime configuration.
#[derive(Debug, Clone)]
pub struct ServerRuntimeConfig {
    /// Path to the composite WASM component.
    pub component_path: PathBuf,
    /// HTTP bind address.
    pub bind_addr: std::net::SocketAddr,
    /// The peer `client_id` the server expects on the SSE control channel.
    pub client_id: String,
    /// Optional session id used in outbound event envelopes.
    pub session_id: Option<String>,
    /// Default-deny permission gate policy.
    pub policy: GatePolicy,
    /// Named LLM providers; `default_provider` selects the loop's provider.
    pub providers: std::collections::HashMap<String, LlmProviderSpec>,
    /// Name of the default LLM provider.
    pub default_provider: String,
    /// Server tools registered via `--server-tool` (registration = grant).
    pub server_tools: Vec<ServerToolSpec>,
    /// MCP server connections, always executed server-side.
    pub mcp_servers: Vec<ServerConfig>,
    /// Cap on total bytes stored via `save-state`/`load-state`.
    pub storage_capacity_bytes: usize,
    /// TTL for server-initiated requests awaiting a POST-back.
    pub pending_ttl: Duration,
}

impl Default for ServerRuntimeConfig {
    fn default() -> Self {
        Self {
            component_path: PathBuf::from("dist/antikythera-sdk.wasm"),
            bind_addr: "127.0.0.1:8787".parse().expect("valid default bind addr"),
            client_id: "antikythera-client".to_string(),
            session_id: None,
            policy: GatePolicy::deny_all(),
            providers: std::collections::HashMap::new(),
            default_provider: "stub".to_string(),
            server_tools: Vec::new(),
            mcp_servers: Vec::new(),
            storage_capacity_bytes: 1024 * 1024,
            pending_ttl: Duration::from_secs(60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_denies_every_tool() {
        let policy = GatePolicy::deny_all();
        let err = policy.check_tool(Destination::Local, "echo").unwrap_err();
        assert!(err.starts_with("permission: "), "got: {err}");
        assert_eq!(err, "permission: tool 'echo' not in allowlist");
        assert!(policy.check_tool(Destination::Remote, "echo").is_err());
        assert!(policy.check_tool(Destination::Mcp, "echo").is_err());
    }

    #[test]
    fn allowlist_grants_only_listed_tool() {
        let mut policy = GatePolicy::deny_all();
        policy.allow_tool(Destination::Local, "echo");
        assert!(policy.check_tool(Destination::Local, "echo").is_ok());
        let err = policy.check_tool(Destination::Local, "rm").unwrap_err();
        assert!(err.starts_with("permission: "));
        // the allowlist is per destination
        assert!(policy.check_tool(Destination::Remote, "echo").is_err());
    }

    #[test]
    fn hook_gate_denies_by_default() {
        let policy = GatePolicy::deny_all();
        let err = policy.check_hook(HookName::DecideAction).unwrap_err();
        assert!(err.starts_with("permission: "));
        assert_eq!(err, "permission: hook 'decide-action' not allowed");

        let mut policy = GatePolicy::deny_all();
        policy.allow_hook(HookName::DecideAction);
        assert!(policy.check_hook(HookName::DecideAction).is_ok());
        assert!(policy.check_hook(HookName::PrepareTurn).is_err());
    }

    #[test]
    fn llm_quota_denies_at_limit() {
        let mut policy = GatePolicy::deny_all();
        policy.set_llm_quota(3);
        assert!(policy.check_llm(0).is_ok());
        assert!(policy.check_llm(2).is_ok());
        let err = policy.check_llm(3).unwrap_err();
        assert!(err.starts_with("permission: "));
        assert_eq!(err, "permission: llm quota exceeded");

        // no quota = unlimited
        let policy = GatePolicy::deny_all();
        assert!(policy.check_llm(10_000).is_ok());
    }

    #[test]
    fn hook_name_kebab_case() {
        assert_eq!(HookName::PrepareTurn.as_str(), "prepare-turn");
        assert_eq!(HookName::DecideAction.as_str(), "decide-action");
        assert_eq!(HookName::HandleToolResult.as_str(), "handle-tool-result");
    }

    #[test]
    fn server_tool_spec_parses_name_before_first_colon() {
        let spec = ServerToolSpec::parse("server_echo:{\"ok\":true}").unwrap();
        assert_eq!(spec.name, "server_echo");
        assert_eq!(spec.response_json, serde_json::json!({"ok": true}));

        // response-json may itself contain colons; only the FIRST colon splits
        let spec = ServerToolSpec::parse("deep:{\"a:b\":{\"b\":1}}").unwrap();
        assert_eq!(spec.name, "deep");
        assert_eq!(spec.response_json, serde_json::json!({"a:b":{"b":1}}));

        // whitespace around the name is trimmed
        let spec = ServerToolSpec::parse("  padded  :[1,2]").unwrap();
        assert_eq!(spec.name, "padded");
        assert_eq!(spec.response_json, serde_json::json!([1, 2]));
    }

    #[test]
    fn server_tool_spec_rejects_missing_colon_and_bad_json() {
        let err = ServerToolSpec::parse("no-colon").unwrap_err();
        assert!(
            err.contains("expected <name>:<response-json>"),
            "got: {err}"
        );

        let err = ServerToolSpec::parse(":{\"ok\":true}").unwrap_err();
        assert!(err.contains("name must not be empty"), "got: {err}");

        let err = ServerToolSpec::parse("broken:not-json").unwrap_err();
        assert!(
            err.contains("invalid --server-tool response-json"),
            "got: {err}"
        );
        assert!(err.contains("broken"), "got: {err}");
    }
}
