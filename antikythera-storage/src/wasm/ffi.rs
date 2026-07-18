//! C FFI exports for WASM storage interop.

use super::WasmStorage;
use crate::config::StorageConfig;

/// Opaque handle for FFI consumers.
pub struct WasmStorageHandle {
    #[allow(dead_code)]
    storage: WasmStorage,
}

/// Create a new storage handle from a TOML config string.
///
/// Returns a pointer to the handle, or null on error.
/// The caller must free with `storage_free`.
///
/// # Safety
/// `config_ptr` must point to valid UTF-8 bytes of length `config_len`.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn storage_new(config_ptr: *const u8, config_len: usize) -> *mut WasmStorageHandle {
    let config_str = match unsafe {
        std::str::from_utf8(std::slice::from_raw_parts(config_ptr, config_len))
    } {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let config = match StorageConfig::from_toml(config_str) {
        Ok(c) => c,
        Err(_) => return std::ptr::null_mut(),
    };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(_) => return std::ptr::null_mut(),
    };

    let storage = match rt.block_on(WasmStorage::new(config)) {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    Box::into_raw(Box::new(WasmStorageHandle { storage }))
}

/// Free a storage handle.
///
/// # Safety
/// `handle` must be a pointer returned by `storage_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn storage_free(handle: *mut WasmStorageHandle) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle)) };
    }
}
