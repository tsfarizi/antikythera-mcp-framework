# antikythera-observability

In-memory observability implementations for the Antikythera Agent SDK.

Migrated from `antikythera-core` to provide a standalone crate for metrics, tracing, and audit implementations.

## Features

- **Metrics**: In-memory metrics exporter with counter, gauge, and histogram support. Includes latency tracking with percentile summaries (p50, p95, p99).
- **Tracing**: In-memory tracing hook with span start/end tracking and correlation support.
- **Audit**: In-memory audit trail for policy decisions, tool executions, and model requests.
- **Facade**: Unified `ObservabilityFacade` that owns all observability implementations and exposes them via port traits.

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `memory` | ✅ | In-memory implementations (metrics, tracing, audit) |
| `full` | ✅ | All features enabled |

## Usage

```rust
use antikythera_observability::{ObservabilityFacade, LatencyTracker};

let facade = ObservabilityFacade::from_config().unwrap();

// Access port trait implementations
if let Some(metrics) = facade.metrics() {
    // metrics implements antikythera_ports::observability::MetricsExporter
}

// Latency tracking
let mut tracker = LatencyTracker::new();
tracker.record_ms(120.0);
tracker.record_ms(240.0);
let summary = tracker.summary();
```
