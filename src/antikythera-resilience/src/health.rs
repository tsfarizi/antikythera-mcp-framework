use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Coarse health classification for a tracked component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// No recent errors; operating normally.
    Healthy,
    /// Non-zero error rate but still functional (error rate < 50 %).
    Degraded,
    /// Half or more of recent calls failed; treat as unavailable.
    Unhealthy,
}

impl HealthStatus {
    fn from_error_rate(rate: f64) -> Self {
        if rate == 0.0 {
            HealthStatus::Healthy
        } else if rate < 0.5 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        }
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Accumulated health metrics for a single named component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub component_id: String,
    pub status: HealthStatus,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub error_rate: f64,
    pub avg_latency_ms: f64,
    pub last_error: Option<String>,
}

impl ComponentHealth {
    fn new(component_id: impl Into<String>) -> Self {
        Self {
            component_id: component_id.into(),
            status: HealthStatus::Healthy,
            total_calls: 0,
            successful_calls: 0,
            error_rate: 0.0,
            avg_latency_ms: 0.0,
            last_error: None,
        }
    }

    /// Record a successful call with `latency_ms` round-trip time.
    pub fn record_success(&mut self, latency_ms: u64) {
        self.total_calls += 1;
        self.successful_calls += 1;
        self.avg_latency_ms = ema(self.avg_latency_ms, latency_ms as f64, self.total_calls);
        self.error_rate = 1.0 - (self.successful_calls as f64 / self.total_calls as f64);
        self.status = HealthStatus::from_error_rate(self.error_rate);
    }

    /// Record a failed call with the error description.
    pub fn record_failure(&mut self, error: impl Into<String>) {
        self.total_calls += 1;
        self.last_error = Some(error.into());
        self.error_rate = 1.0 - (self.successful_calls as f64 / self.total_calls as f64);
        self.status = HealthStatus::from_error_rate(self.error_rate);
    }
}

/// Exponential moving average with alpha = 2 / (min(N, 20) + 1).
fn ema(current: f64, new_sample: f64, n: u64) -> f64 {
    if n <= 1 {
        return new_sample;
    }
    let alpha = 2.0 / (n.min(20) as f64 + 1.0);
    current * (1.0 - alpha) + new_sample * alpha
}

/// Aggregates health metrics across multiple named components.
#[derive(Debug, Default)]
pub struct HealthTracker {
    components: HashMap<String, ComponentHealth>,
}

impl HealthTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful invocation of `component_id` with the measured
    /// latency.
    pub fn record_success(&mut self, component_id: &str, latency_ms: u64) {
        self.components
            .entry(component_id.to_string())
            .or_insert_with(|| ComponentHealth::new(component_id))
            .record_success(latency_ms);
    }

    /// Record a failed invocation of `component_id`.
    pub fn record_failure(&mut self, component_id: &str, error: impl Into<String>) {
        let error_str: String = error.into();
        self.components
            .entry(component_id.to_string())
            .or_insert_with(|| ComponentHealth::new(component_id))
            .record_failure(error_str);
    }

    /// Retrieve the health snapshot for a specific component.
    pub fn health_of(&self, component_id: &str) -> Option<&ComponentHealth> {
        self.components.get(component_id)
    }

    /// Aggregate status: the worst status across all tracked components.
    pub fn overall_status(&self) -> HealthStatus {
        let mut worst = HealthStatus::Healthy;
        for c in self.components.values() {
            match c.status {
                HealthStatus::Unhealthy => return HealthStatus::Unhealthy,
                HealthStatus::Degraded => worst = HealthStatus::Degraded,
                HealthStatus::Healthy => {}
            }
        }
        worst
    }

    /// Reset all accumulated statistics for all components.
    pub fn reset(&mut self) {
        self.components.clear();
    }

    /// Serialise the full component map to a JSON array string.
    pub fn snapshot_json(&self) -> String {
        let snapshot: Vec<&ComponentHealth> = self.components.values().collect();
        serde_json::to_string(&snapshot).unwrap_or_else(|_| "[]".to_string())
    }
}
