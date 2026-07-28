use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metric type emitted by host-facing exporters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricKind {
    Counter,
    Gauge,
    Histogram,
}

/// Metric record captured by observability exporters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricRecord {
    pub name: String,
    pub kind: MetricKind,
    pub value: f64,
    pub timestamp_ms: u64,
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

impl MetricRecord {
    /// Construct a metric record.
    pub fn new(
        name: impl Into<String>,
        kind: MetricKind,
        value: f64,
        attributes: HashMap<String, String>,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            value,
            timestamp_ms: crate::now_unix_ms(),
            attributes,
        }
    }
}
