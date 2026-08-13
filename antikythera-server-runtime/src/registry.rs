//! Union tool registry (R5): the merged set of tool definitions from the
//! server side, the client side, and MCP servers. Each tool has exactly one
//! owner in `{server, client, mcp}`. The loop owner computes the union and
//! pushes it to the runner in a single `register-tools` call.

use std::collections::HashMap;

use crate::wire::ToolDefinition;

/// Owner side of a tool. Tools are locked to one side; the registry is a
/// union and cross-side name collisions are rejected at registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolOwner {
    Server,
    Client,
    Mcp,
}

impl ToolOwner {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolOwner::Server => "server",
            ToolOwner::Client => "client",
            ToolOwner::Mcp => "mcp",
        }
    }
}

/// Routing destination for a tool call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The core side (server) executes it.
    Local,
    /// The peer (client) executes it via SSE request + POST-back.
    Remote,
    /// A third-party MCP server executes it; always server-side.
    Mcp,
}

impl Destination {
    pub fn as_str(&self) -> &'static str {
        match self {
            Destination::Local => "local",
            Destination::Remote => "remote",
            Destination::Mcp => "mcp",
        }
    }
}

impl From<ToolOwner> for Destination {
    fn from(owner: ToolOwner) -> Self {
        match owner {
            ToolOwner::Server => Destination::Local,
            ToolOwner::Client => Destination::Remote,
            ToolOwner::Mcp => Destination::Mcp,
        }
    }
}

#[derive(Debug, Clone)]
struct ToolEntry {
    owner: ToolOwner,
    definition: ToolDefinition,
}

/// Merged registry with collision detection. Registration replaces an
/// existing entry of the SAME owner; a different owner is an explicit error.
#[derive(Debug, Default)]
pub struct UnionRegistry {
    tools: HashMap<String, ToolEntry>,
}

impl UnionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool definition under an owner.
    ///
    /// Errors with the canonical R5 message when the name is already owned
    /// by a different side:
    /// `tool registry: name collision for tool '<name>' (owners <a>, <b>)`.
    pub fn register(&mut self, owner: ToolOwner, definition: ToolDefinition) -> Result<(), String> {
        let name = definition.name.clone();
        if let Some(existing) = self.tools.get(&name)
            && existing.owner != owner
        {
            return Err(format!(
                "tool registry: name collision for tool '{}' (owners {}, {})",
                name,
                existing.owner.as_str(),
                owner.as_str()
            ));
        }
        self.tools.insert(name, ToolEntry { owner, definition });
        Ok(())
    }

    pub fn owner_of(&self, name: &str) -> Option<ToolOwner> {
        self.tools.get(name).map(|e| e.owner)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Definitions owned by `owner`, sorted by name for determinism.
    pub fn definitions_for(&self, owner: ToolOwner) -> Vec<ToolDefinition> {
        let mut defs: Vec<ToolDefinition> = self
            .tools
            .values()
            .filter(|e| e.owner == owner)
            .map(|e| e.definition.clone())
            .collect();
        defs.sort_by(|a, b| a.name.cmp(&b.name));
        defs
    }

    /// The full union, sorted by name for determinism.
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        let mut defs: Vec<ToolDefinition> =
            self.tools.values().map(|e| e.definition.clone()).collect();
        defs.sort_by(|a, b| a.name.cmp(&b.name));
        defs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(name: &str) -> ToolDefinition {
        ToolDefinition::simple(name, "test tool")
    }

    #[test]
    fn cross_side_collision_is_rejected() {
        let mut registry = UnionRegistry::new();
        registry
            .register(ToolOwner::Server, def("get_current_time"))
            .unwrap();
        let err = registry
            .register(ToolOwner::Client, def("get_current_time"))
            .unwrap_err();
        assert_eq!(
            err,
            "tool registry: name collision for tool 'get_current_time' (owners server, client)"
        );
        let err = registry
            .register(ToolOwner::Mcp, def("get_current_time"))
            .unwrap_err();
        assert_eq!(
            err,
            "tool registry: name collision for tool 'get_current_time' (owners server, mcp)"
        );
        // the original owner is untouched
        assert_eq!(
            registry.owner_of("get_current_time"),
            Some(ToolOwner::Server)
        );
    }

    #[test]
    fn same_owner_reregistration_replaces() {
        let mut registry = UnionRegistry::new();
        registry.register(ToolOwner::Server, def("echo")).unwrap();
        let updated = ToolDefinition {
            name: "echo".to_string(),
            title: Some("New Echo".to_string()),
            description: "updated".to_string(),
            parameters: Vec::new(),
            input_schema: None,
            output_schema: None,
        };
        registry.register(ToolOwner::Server, updated).unwrap();
        let defs = registry.definitions_for(ToolOwner::Server);
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].title.as_deref(), Some("New Echo"));
    }

    #[test]
    fn owner_resolution_and_union() {
        let mut registry = UnionRegistry::new();
        registry.register(ToolOwner::Server, def("a")).unwrap();
        registry.register(ToolOwner::Client, def("b")).unwrap();
        registry.register(ToolOwner::Mcp, def("c")).unwrap();

        assert_eq!(registry.owner_of("a"), Some(ToolOwner::Server));
        assert_eq!(registry.owner_of("b"), Some(ToolOwner::Client));
        assert_eq!(registry.owner_of("c"), Some(ToolOwner::Mcp));
        assert_eq!(registry.owner_of("nope"), None);

        let union = registry.definitions();
        assert_eq!(
            union.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
        assert_eq!(registry.definitions_for(ToolOwner::Server).len(), 1);
        assert_eq!(registry.definitions_for(ToolOwner::Client).len(), 1);
        assert_eq!(registry.definitions_for(ToolOwner::Mcp).len(), 1);
    }

    #[test]
    fn destination_from_owner() {
        assert_eq!(Destination::from(ToolOwner::Server), Destination::Local);
        assert_eq!(Destination::from(ToolOwner::Client), Destination::Remote);
        assert_eq!(Destination::from(ToolOwner::Mcp), Destination::Mcp);
    }
}
