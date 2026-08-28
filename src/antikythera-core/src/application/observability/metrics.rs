//! Metrics types and in-memory exporter.
//!
//! Canonical source: `antikythera_observability::metrics`.
//! Re-exported here for backward compatibility.

pub use antikythera_observability::metrics::{
    InMemoryMetricsExporter, LatencySummary, LatencyTracker, MetricKind, MetricRecord,
    MetricsExporter, percentile,
};
