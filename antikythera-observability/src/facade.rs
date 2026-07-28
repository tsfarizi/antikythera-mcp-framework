use crate::audit::AuditTrail;
use crate::error::ObservabilityError;
use crate::metrics::InMemoryMetricsExporter;
use crate::tracing::InMemoryTracingHook;

/// Unified facade that owns observability implementations.
pub struct ObservabilityFacade {
    metrics: InMemoryMetricsExporter,
    tracer: InMemoryTracingHook,
    audit: AuditTrail,
}

impl ObservabilityFacade {
    /// Create a facade with default in-memory implementations.
    pub fn from_config() -> Result<Self, ObservabilityError> {
        Ok(Self {
            metrics: InMemoryMetricsExporter::new(),
            tracer: InMemoryTracingHook::new(),
            audit: AuditTrail::new(),
        })
    }

    pub fn metrics(&self) -> Option<&(dyn antikythera_ports::observability::MetricsExporter + '_)> {
        Some(&self.metrics)
    }

    pub fn tracer(&self) -> Option<&(dyn antikythera_ports::observability::TracingHook + '_)> {
        Some(&self.tracer)
    }

    pub fn audit(&self) -> Option<&(dyn antikythera_ports::observability::AuditSink + '_)> {
        Some(&self.audit)
    }
}
