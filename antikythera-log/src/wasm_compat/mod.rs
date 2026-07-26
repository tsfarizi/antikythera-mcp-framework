//! WASM-compatible time abstraction.
//!
//! Provides platform-safe time functions that work across:
//! - **Native** (`not(target_arch = "wasm32")`) — delegates to `chrono::Utc::now()`
//! - **Browser** (`wasm32-unknown-unknown` + `wasm` feature) — delegates to `js_sys::Date::now()`
//! - **WASI** (`wasm32-wasip1`) — delegates to `chrono::Utc::now()` (has system clock)
//!
//! All functions return identical semantics: Unix timestamps in milliseconds
//! and RFC 3339 formatted strings. The mechanism differs by target to avoid
//! `SystemTime::now()` panics in browser WASM.

// Browser WASM with wasm feature: use js_sys (no SystemTime panic)
#[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "wasm"))]
mod browser;

#[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "wasm"))]
pub use browser::*;

// All other targets (native, wasm32-unknown-unknown without wasm, wasm32-wasip1): use chrono
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown", feature = "wasm")))]
mod native;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown", feature = "wasm")))]
pub use native::*;
