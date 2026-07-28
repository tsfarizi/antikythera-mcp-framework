use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Minimal span context used by tracing hooks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceSpanContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub correlation_id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

impl TraceSpanContext {
    pub fn new(
        trace_id: impl Into<String>,
        span_id: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            parent_span_id: None,
            correlation_id: None,
            name: name.into(),
            attributes: HashMap::new(),
        }
    }

    pub fn with_parent(mut self, parent_span_id: impl Into<String>) -> Self {
        self.parent_span_id = Some(parent_span_id.into());
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// Span status classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceStatus {
    Ok,
    Error,
}

/// Local sync tracing hook — backward-compatible with core's internal usage.
pub trait TracingHook: Send + Sync {
    fn on_span_start(&self, span: TraceSpanContext);
    fn on_span_end(&self, span: TraceSpanContext, status: TraceStatus);
}

/// In-memory tracing hook used by tests.
#[derive(Debug, Clone, Default)]
pub struct InMemoryTracingHook {
    started: Arc<Mutex<Vec<TraceSpanContext>>>,
    ended: Arc<Mutex<Vec<(TraceSpanContext, TraceStatus)>>>,
}

impl InMemoryTracingHook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn started_spans(&self) -> Vec<TraceSpanContext> {
        self.started
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|e| {
                ::tracing::warn!(
                    "InMemoryTracingHook started lock poisoned in started_spans: {}",
                    e
                );
                Vec::new()
            })
    }

    pub fn ended_spans(&self) -> Vec<(TraceSpanContext, TraceStatus)> {
        self.ended
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|e| {
                ::tracing::warn!(
                    "InMemoryTracingHook ended lock poisoned in ended_spans: {}",
                    e
                );
                Vec::new()
            })
    }
}

impl TracingHook for InMemoryTracingHook {
    fn on_span_start(&self, span: TraceSpanContext) {
        match self.started.lock() {
            Ok(mut guard) => guard.push(span),
            Err(e) => {
                ::tracing::warn!(
                    "InMemoryTracingHook started lock poisoned in on_span_start: {}",
                    e
                );
            }
        }
    }

    fn on_span_end(&self, span: TraceSpanContext, status: TraceStatus) {
        match self.ended.lock() {
            Ok(mut guard) => guard.push((span, status)),
            Err(e) => {
                ::tracing::warn!(
                    "InMemoryTracingHook ended lock poisoned in on_span_end: {}",
                    e
                );
            }
        }
    }
}

#[async_trait]
impl antikythera_ports::observability::TracingHook for InMemoryTracingHook {
    async fn start_span(&self, name: &str, attributes: Vec<(String, String)>) -> String {
        let span_id = uuid::Uuid::new_v4().to_string();
        let trace_id = uuid::Uuid::new_v4().to_string();
        let mut span = TraceSpanContext::new(trace_id, span_id.clone(), name);
        for (k, v) in attributes {
            span = span.with_attribute(k, v);
        }
        self.on_span_start(span);
        span_id
    }

    async fn end_span(&self, span_id: &str, status: &str) {
        let trace_id = uuid::Uuid::new_v4().to_string();
        let span = TraceSpanContext::new(trace_id, span_id.to_string(), "");
        let trace_status = match status {
            "ok" => TraceStatus::Ok,
            _ => TraceStatus::Error,
        };
        self.on_span_end(span, trace_status);
    }
}
