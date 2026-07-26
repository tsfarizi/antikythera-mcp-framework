use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Policy decision event types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEventType {
    ContextPolicyApplied,
    ContextPolicyOverride,
    ToolAccessDenied,
    ToolAccessGranted,
    RateLimitTriggered,
    TimeoutTriggered,
    RetryPolicyActivated,
    HealthCheckFailed,
    CustomPolicy,
}

/// Audit event for policy decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAuditEvent {
    pub timestamp: String,
    pub correlation_id: Option<String>,
    pub session_id: Option<String>,
    pub event_type: PolicyEventType,
    pub policy_name: String,
    pub decision: String,
    pub reason: String,
    pub resource: Option<String>,
    pub caller: Option<HashMap<String, String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl PolicyAuditEvent {
    pub fn new(
        correlation_id: Option<String>,
        session_id: Option<String>,
        event_type: PolicyEventType,
        policy_name: impl Into<String>,
        decision: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            correlation_id,
            session_id,
            event_type,
            policy_name: policy_name.into(),
            decision: decision.into(),
            reason: reason.into(),
            resource: None,
            caller: None,
            metadata: None,
        }
    }

    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    pub fn with_caller(mut self, caller: HashMap<String, String>) -> Self {
        self.caller = Some(caller);
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Policy audit event sink — implement this to capture events.
pub trait PolicyAuditSink: Send + Sync {
    fn record_event(&self, event: PolicyAuditEvent);
}

/// No-op audit sink (discards all events).
pub struct NoOpAuditSink;

impl PolicyAuditSink for NoOpAuditSink {
    fn record_event(&self, _event: PolicyAuditEvent) {}
}

/// In-memory audit sink for testing.
#[derive(Debug, Clone)]
pub struct InMemoryAuditSink {
    events: std::sync::Arc<std::sync::Mutex<Vec<PolicyAuditEvent>>>,
}

impl InMemoryAuditSink {
    pub fn new() -> Self {
        Self {
            events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn snapshot(&self) -> Vec<PolicyAuditEvent> {
        self.events
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.events.lock() {
            guard.clear();
        }
    }
}

impl Default for InMemoryAuditSink {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyAuditSink for InMemoryAuditSink {
    fn record_event(&self, event: PolicyAuditEvent) {
        if let Ok(mut guard) = self.events.lock() {
            guard.push(event);
        }
    }
}
