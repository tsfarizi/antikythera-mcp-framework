//! Port: Observability
//!
//! Application defines these traits. Infrastructure implements them.

use async_trait::async_trait;

/// Metrics export abstraction.
#[async_trait]
pub trait MetricsExporter: Send + Sync {
    /// Record a metric.
    async fn record_metric(
        &self,
        name: &str,
        kind: &str,
        value: f64,
        attributes: Vec<(String, String)>,
    );

    /// Flush pending metrics.
    async fn flush(&self) -> Result<(), String>;
}

/// Tracing abstraction.
#[async_trait]
pub trait TracingHook: Send + Sync {
    /// Start a new trace span.
    async fn start_span(&self, name: &str, attributes: Vec<(String, String)>) -> String;

    /// End a trace span.
    async fn end_span(&self, span_id: &str, status: &str);
}

/// Audit logging abstraction.
#[async_trait]
pub trait AuditSink: Send + Sync {
    /// Record an audit event.
    async fn record_event(
        &self,
        category: &str,
        action: &str,
        allowed: bool,
        details: Vec<(String, String)>,
    );
}
