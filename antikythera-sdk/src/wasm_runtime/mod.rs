//! Wasmtime-based WASM agent runner (SDK-owned).
//!
//! Moved from antikythera-core infrastructure. The WASM runtime is an
//! SDK concern — it bridges the WASM component boundary.

#[cfg(feature = "wasm-sandbox")]
pub use runner::WasmAgentRunner;
#[cfg(feature = "wasm-sandbox")]
pub use runner::WasmRuntimeError;

#[cfg(feature = "wasm-sandbox")]
mod runner;
