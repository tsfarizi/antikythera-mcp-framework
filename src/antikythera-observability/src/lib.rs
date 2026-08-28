pub mod audit;
pub mod error;
pub mod facade;
pub mod metrics;
pub mod telemetry;
pub mod tracing;

pub use audit::{AuditCategory, AuditRecord, AuditTrail};
pub use error::ObservabilityError;
pub use facade::ObservabilityFacade;
pub use metrics::{
    InMemoryMetricsExporter, LatencySummary, LatencyTracker, MetricKind, MetricRecord,
    MetricsExporter, percentile,
};
pub use telemetry::{CallerContext, TelemetryEvent};
pub use tracing::{InMemoryTracingHook, TraceSpanContext, TraceStatus, TracingHook};

pub(crate) fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
