//! Tracing types and in-memory hook.
//!
//! Canonical source for trace types: `antikythera_observability::tracing`.
//! `ObservabilityHook` and `NoOpObservabilityHook` remain here as core-specific.

use std::collections::HashMap;

pub use antikythera_observability::tracing::{
    InMemoryTracingHook, TraceSpanContext, TraceStatus, TracingHook,
};

use super::telemetry::TelemetryEvent;

/// Observability hook — implement to receive telemetry events.
pub trait ObservabilityHook: Send + Sync {
    /// Record a telemetry event.
    fn record_event(&self, event: TelemetryEvent);

    /// Record a metric (counter, gauge, histogram).
    fn record_metric(&self, name: &str, value: f64, attributes: &HashMap<String, String>) {
        let _ = (name, value, attributes);
    }
}

/// No-op observability hook (discards all events).
pub struct NoOpObservabilityHook;

impl ObservabilityHook for NoOpObservabilityHook {
    fn record_event(&self, _event: TelemetryEvent) {}
}
