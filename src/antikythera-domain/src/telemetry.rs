//! Telemetry types for structured observability.
//!
//! These types are the canonical source for caller context and telemetry events.
//! `antikythera-core` and `antikythera-observability` re-export from here.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Caller context — propagated through all framework operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CallerContext {
    /// Unique ID for this request/session (for end-to-end tracing)
    pub correlation_id: Option<String>,
    /// User ID or service principal
    pub user_id: Option<String>,
    /// Tenant or organization ID
    pub tenant_id: Option<String>,
    /// Request source (CLI, REST, gRPC, WASM, etc.)
    pub source: Option<String>,
    /// Custom metadata propagated by the host
    pub metadata: Option<HashMap<String, String>>,
}

impl CallerContext {
    /// Create a new caller context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set correlation ID for tracing.
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Set user ID.
    pub fn with_user_id(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    /// Set tenant ID.
    pub fn with_tenant_id(mut self, id: impl Into<String>) -> Self {
        self.tenant_id = Some(id.into());
        self
    }

    /// Set request source.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Add custom metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        if let Some(ref mut meta) = self.metadata {
            meta.insert(key.into(), value.into());
        }
        self
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Returns a correlation ID if present, otherwise generates a deterministic
    /// fallback using timestamp-based entropy.
    pub fn ensure_correlation_id(&mut self) -> String {
        if let Some(value) = self.correlation_id.clone() {
            return value;
        }

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let generated = format!("corr-{}", now_ms);
        self.correlation_id = Some(generated.clone());
        generated
    }
}

/// Telemetry event — structured observability data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Event type (e.g., "agent_step", "tool_call", "llm_request")
    pub event_type: String,
    /// Correlation ID for tracing
    pub correlation_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Timestamp (Unix epoch milliseconds)
    pub timestamp_ms: u64,
    /// Event-specific attributes
    pub attributes: HashMap<String, serde_json::Value>,
}

impl TelemetryEvent {
    /// Create a new telemetry event.
    pub fn new(
        event_type: impl Into<String>,
        correlation_id: Option<String>,
        session_id: Option<String>,
    ) -> Self {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            event_type: event_type.into(),
            correlation_id,
            session_id,
            timestamp_ms: now_ms,
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute.
    pub fn with_attribute(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Build flat string attributes suitable for metric exporters.
    pub fn metric_attributes(&self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        attrs.insert("event_type".to_string(), self.event_type.clone());

        if let Some(correlation_id) = &self.correlation_id {
            attrs.insert("correlation_id".to_string(), correlation_id.clone());
        }
        if let Some(session_id) = &self.session_id {
            attrs.insert("session_id".to_string(), session_id.clone());
        }

        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caller_context_default() {
        let ctx = CallerContext::new();
        assert!(ctx.correlation_id.is_none());
        assert!(ctx.user_id.is_none());
    }

    #[test]
    fn caller_context_builder() {
        let ctx = CallerContext::new()
            .with_correlation_id("corr-1")
            .with_user_id("user-1")
            .with_tenant_id("tenant-1")
            .with_source("cli")
            .with_metadata("key", "value");

        assert_eq!(ctx.correlation_id.as_deref(), Some("corr-1"));
        assert_eq!(ctx.user_id.as_deref(), Some("user-1"));
        assert_eq!(ctx.tenant_id.as_deref(), Some("tenant-1"));
        assert_eq!(ctx.source.as_deref(), Some("cli"));
        assert_eq!(ctx.metadata.as_ref().unwrap().get("key").unwrap(), "value");
    }

    #[test]
    fn caller_context_json_roundtrip() {
        let ctx = CallerContext::new()
            .with_correlation_id("corr-1")
            .with_user_id("user-1");
        let json = ctx.to_json().unwrap();
        let restored = CallerContext::from_json(&json).unwrap();
        assert_eq!(ctx, restored);
    }

    #[test]
    fn telemetry_event_builder() {
        let event = TelemetryEvent::new("tool_call", Some("corr-1".into()), Some("s1".into()))
            .with_attribute("tool", serde_json::json!("fs.read"));

        assert_eq!(event.event_type, "tool_call");
        assert_eq!(event.correlation_id.as_deref(), Some("corr-1"));
        assert_eq!(event.session_id.as_deref(), Some("s1"));
        assert_eq!(
            event.attributes.get("tool").unwrap(),
            &serde_json::json!("fs.read")
        );
    }

    #[test]
    fn telemetry_event_metric_attributes() {
        let event = TelemetryEvent::new("agent_step", Some("corr-1".into()), Some("s1".into()));
        let attrs = event.metric_attributes();
        assert_eq!(attrs.get("event_type").unwrap(), "agent_step");
        assert_eq!(attrs.get("correlation_id").unwrap(), "corr-1");
        assert_eq!(attrs.get("session_id").unwrap(), "s1");
    }

    #[test]
    fn telemetry_event_json_roundtrip() {
        let event = TelemetryEvent::new("test", None, None);
        let json = event.to_json().unwrap();
        let restored: TelemetryEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.event_type, restored.event_type);
    }
}
