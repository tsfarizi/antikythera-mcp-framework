//! WASM↔Core type bridging trait.
//!
//! Implemented by types that cross the WASM boundary. The `WasmBridge` derive
//! in `antikythera-macros` generates an `impl` of this trait from the type's
//! `Serialize`/`Deserialize` impls, so production types only need the trait
//! in scope plus the derive:
//!
//! ```ignore
//! use antikythera_core::WasmBridge;
//!
//! #[derive(serde::Serialize, serde::Deserialize, antikythera_macros::WasmBridge)]
//! pub struct ToolCall {
//!     pub name: String,
//! }
//! ```

/// Trait for types that have WASM bridge support.
///
/// Types implementing this trait can be converted between native Rust
/// representations and WASM-compatible JSON representations.
pub trait WasmBridge: Sized {
    /// The name of this type on the WASM side.
    const WASM_TYPE_NAME: &'static str;

    /// The bridge target this type is bound to (e.g. `"wasm"`).
    const BRIDGE_TARGET: &'static str;

    /// Convert from the core representation to a JSON-friendly form.
    fn to_json_value(&self) -> Result<serde_json::Value, String>;

    /// Convert from a JSON-friendly form back to the core representation.
    fn from_json_value(value: serde_json::Value) -> Result<Self, String>;
}
