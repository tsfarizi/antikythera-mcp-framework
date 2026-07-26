pub mod context_window;
pub mod health;
pub mod policy;
pub mod policy_audit;
pub mod retry;

pub use context_window::{ContextWindowPolicy, TokenEstimator, prune_messages};
pub use health::{ComponentHealth, HealthStatus, HealthTracker};
pub use policy::{ResilienceConfig, RetryPolicy, TimeoutPolicy};
pub use policy_audit::{
    InMemoryAuditSink, NoOpAuditSink, PolicyAuditEvent, PolicyAuditSink, PolicyEventType,
};
pub use retry::{with_retry, with_retry_if};

use antikythera_domain::types::ChatMessage;

/// Unified facade that owns a [`ResilienceConfig`] and a [`HealthTracker`].
#[derive(Debug, Default)]
pub struct ResilienceManager {
    config: ResilienceConfig,
    health: HealthTracker,
}

impl ResilienceManager {
    /// Create a manager with default policies.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a manager with a specific [`ResilienceConfig`].
    pub fn with_config(config: ResilienceConfig) -> Self {
        Self {
            config,
            health: HealthTracker::new(),
        }
    }

    pub fn config(&self) -> &ResilienceConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: ResilienceConfig) {
        self.config = config;
    }

    pub fn health(&self) -> &HealthTracker {
        &self.health
    }

    pub fn health_mut(&mut self) -> &mut HealthTracker {
        &mut self.health
    }

    /// Return the current config as a JSON string.
    pub fn get_config_json(&self) -> String {
        serde_json::to_string(&self.config).unwrap_or_else(|_| "{}".to_string())
    }

    /// Overwrite the config from a JSON string.
    pub fn set_config_from_json(&mut self, config_json: &str) -> Result<bool, String> {
        let config: ResilienceConfig =
            serde_json::from_str(config_json).map_err(|e| e.to_string())?;
        self.config = config;
        Ok(true)
    }

    /// Return a JSON array of all tracked component health snapshots.
    pub fn get_health_json(&self) -> String {
        self.health.snapshot_json()
    }

    /// Clear all accumulated health statistics.
    pub fn reset_health(&mut self) {
        self.health.reset();
    }

    /// Estimate the token count for `text`.
    pub fn estimate_tokens(text: &str) -> u32 {
        TokenEstimator::estimate_text(text) as u32
    }

    /// Prune a JSON-encoded message array to fit within `max_tokens`.
    pub fn prune_messages_json(
        messages_json: &str,
        max_tokens: u32,
        reserve_tokens: u32,
    ) -> Result<String, String> {
        let messages: Vec<ChatMessage> =
            serde_json::from_str(messages_json).map_err(|e| e.to_string())?;
        let policy = ContextWindowPolicy {
            max_tokens: max_tokens as usize,
            reserve_for_response: reserve_tokens as usize,
            min_history_messages: 2,
        };
        let pruned = prune_messages(&messages, &policy);
        serde_json::to_string(&pruned).map_err(|e| e.to_string())
    }
}
