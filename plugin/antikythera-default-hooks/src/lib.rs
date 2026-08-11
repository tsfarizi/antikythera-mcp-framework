//! # Antikythera Default Hooks
//!
//! No-op passthrough implementation of the `logic-hooks` WIT interface
//! (`antikythera:agent-sdk/logic-hooks`). Every hook returns the single-key
//! object `{"passthrough": true}`, the SDK's default-behavior signal, so a
//! composite built from this component behaves exactly like the SDK alone.
//!
//! ## WASM Integration
//!
//! When compiled with the `component` feature, the `component` module exports
//! the `logic-hooks-component` world. The crate itself carries no business
//! logic; it is the reference default for `wasm-tools compose` wiring.

/// WIT export layer for the component world (`antikythera:agent-sdk/logic-hooks-component`)
#[cfg(feature = "component")]
pub mod component;
