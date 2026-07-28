//! Tool handler function types.
//!
//! A `ToolHandlerFn` is a synchronous function that takes tool arguments
//! and returns a result. This is intentionally synchronous to support
//! WASM execution where async runtime may not be available.

use serde_json::Value;

/// Synchronous tool handler function.
///
/// Takes JSON arguments and returns a JSON result or an error message.
/// The handler must be pure and fast — no I/O, no network, no blocking.
///
/// For native hosts that need async I/O, wrap the handler with a channel
/// bridge or use the `ToolRunner`'s async `execute_async` method.
pub type ToolHandlerFn = fn(arguments: &Value) -> Result<Value, String>;

/// Dynamic tool handler trait for more complex scenarios.
///
/// Use this when the handler needs captured state (e.g., database connection,
/// API client). The trait object is stored behind `Arc` in the `ToolRunner`.
pub trait ToolHandler: Send + Sync {
    fn call(&self, arguments: &Value) -> Result<Value, String>;
}

/// Blanket implementation for `ToolHandlerFn`.
impl ToolHandler for ToolHandlerFn {
    fn call(&self, arguments: &Value) -> Result<Value, String> {
        self(arguments)
    }
}
