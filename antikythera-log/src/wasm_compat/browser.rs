//! Browser WASM time implementation — delegates to JavaScript `Date.now()`.
//!
//! `js_sys::Date::now()` returns milliseconds since Unix epoch,
//! matching the semantics of `chrono::Utc::now().timestamp_millis()`.

/// Current Unix timestamp in milliseconds.
///
/// Calls JavaScript `Date.now()` via `js_sys`. Does not panic
/// in `wasm32-unknown-unknown` unlike `chrono::Utc::now()`.
pub fn now_unix_ms() -> i64 {
    js_sys::Date::now() as i64
}

/// Current time as RFC 3339 string.
///
/// Uses `js_sys::Date::new_0()` to create a Date object for the current
/// moment, then calls `to_iso_string()` for reliable RFC 3339 formatting.
pub fn now_rfc3339() -> String {
    let dt = js_sys::Date::new_0();
    dt.to_iso_string().into()
}

/// Current Unix timestamp in nanoseconds.
///
/// Constructed from millisecond-precision `Date.now()` — the lower
/// 3 digits of nanoseconds will always be zero.
pub fn now_timestamp_nanos() -> i64 {
    js_sys::Date::now() as i64 * 1_000_000
}
