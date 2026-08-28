//! WASI-compatible time abstraction.
//!
//! All targets delegate to `chrono::Utc::now()`. Under `wasm32-wasip2`
//! this resolves to `wasi:clocks/wall-clock` via the WASI 0.2 shim;
//! when transpiled with `jco` the wall-clock import is stubbed to
//! `Date.now()` in JS.

mod native;

pub use native::*;
