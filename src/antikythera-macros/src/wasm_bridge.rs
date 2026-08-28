//! WasmBridge derive: WASM↔Core type bridging.
//!
//! Converts between native Rust types and their WASM-compatible representations,
//! handling serialization boundaries and memory ownership transfer.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::attr_utils::parse_meta_string;

pub(crate) fn impl_wasm_bridge(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let type_name_str = name.to_string();

    let target = parse_bridge_target(&input.attrs)?;

    let expanded = quote! {
        impl WasmBridge for #name {
            const WASM_TYPE_NAME: &'static str = #type_name_str;
            const BRIDGE_TARGET: &'static str = #target;

            fn to_json_value(&self) -> Result<serde_json::Value, String> {
                serde_json::to_value(self).map_err(|e| e.to_string())
            }

            fn from_json_value(value: serde_json::Value) -> Result<Self, String> {
                serde_json::from_value(value).map_err(|e| e.to_string())
            }
        }
    };

    Ok(expanded)
}

/// Extract the `target` value from `#[bridge(target = "...")]`.
///
/// Defaults to `"wasm"` when the `#[bridge(...)]` attribute is absent. A
/// declared but malformed attribute — empty, unknown argument, malformed
/// literal, or conflicting declarations — is a `syn::Error` at the derive
/// site instead of a silent default.
fn parse_bridge_target(attrs: &[syn::Attribute]) -> syn::Result<String> {
    let mut target: Option<String> = None;

    for attr in attrs {
        if !attr.path().is_ident("bridge") {
            continue;
        }
        if target.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "WasmBridge allows only one #[bridge(...)] attribute",
            ));
        }

        let mut found_target = false;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("target") {
                if found_target {
                    return Err(meta.error("duplicate `target` in #[bridge(...)] attribute"));
                }
                found_target = true;
                target = Some(parse_meta_string(&meta, "target")?);
            } else {
                return Err(meta.error("unknown bridge attribute, expected `target`"));
            }
            Ok(())
        })?;

        if !found_target {
            return Err(syn::Error::new_spanned(
                attr,
                "expected `target` in #[bridge(...)] attribute",
            ));
        }
    }

    Ok(target.unwrap_or_else(|| "wasm".to_string()))
}
