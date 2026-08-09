//! PortValidate derive: compile-time validation for port trait implementations.
//!
//! Generates a compile-time assertion that the annotated struct implements
//! the specified port trait. If any required method is missing, the compiler
//! will produce an error at the derive site.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn impl_port_validate(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;
    let trait_name = extract_implements_trait(input)?;

    let expanded = quote! {
        const _: () = {
            fn _assert_port<T: ?Sized + #trait_name>() {}
            fn _assert_impl() {
                _assert_port::<#struct_name>();
            }
        };
    };

    Ok(expanded)
}

/// Extract the trait name from `#[implements(TraitName)]`.
/// Supports simple names (`ModelClient`), qualified paths (`traits::Storage`),
/// and traits with lifetime parameters (`Processor<'_>`).
fn extract_implements_trait(input: &DeriveInput) -> syn::Result<syn::Path> {
    for attr in &input.attrs {
        if !attr.path().is_ident("implements") {
            continue;
        }

        return attr.parse_args::<syn::Path>().map_err(|_| {
            syn::Error::new_spanned(
                attr,
                "expected `#[implements(TraitPath)]` where TraitPath is a valid trait path",
            )
        });
    }

    Err(syn::Error::new_spanned(
        input,
        "PortValidate requires `#[implements(TraitName)]` attribute",
    ))
}
