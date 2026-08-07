//! # Antikythera Macros
//!
//! Proc macros for compile-time reflection and type-safe code generation.
//!
//! ## Available Derives
//!
//! - `ToolDef` — Generate MCP tool definitions from struct field metadata
//! - `WasmBridge` — Generate WASM↔Core type bridging
//! - `JsonSchema` — Generate JSON Schema from struct definitions
//! - `FsmComplete` — Validate FSM transition completeness

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Lit};

/// Derive macro that generates MCP tool definitions from struct field metadata.
///
/// # Example
///
/// ```ignore
/// #[derive(ToolDef, Serialize)]
/// #[tool(name = "get_weather", description = "Get weather for a city")]
/// struct WeatherTool {
///     #[tool_param(description = "City name")]
///     city: String,
///
///     #[tool_param(description = "Units", required = false)]
///     units: Option<String>,
/// }
/// ```
///
/// Generates:
/// - `TOOL_NAME: &str` — the tool name constant
/// - `TOOL_DESCRIPTION: &str` — the tool description constant
/// - `json_schema() -> serde_json::Value` — the JSON Schema for input parameters
/// - `definition() -> serde_json::Value` — the full MCP tool definition
#[proc_macro_derive(ToolDef, attributes(tool, tool_param))]
pub fn derive_tool_def(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_tool_def(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn impl_tool_def(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;

    // Extract #[tool(name = "...", description = "...")] from the struct.
    let (tool_name, tool_description) = extract_tool_attrs(input)?;

    // Extract fields from the struct.
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "ToolDef can only be derived on structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "ToolDef can only be derived on structs",
            ));
        }
    };

    // Collect field metadata: (field_name_str, description, json_type, required).
    let mut field_meta = Vec::new();
    let mut required_names = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        let (param_description, param_required) = extract_tool_param_attrs(field)?;
        let is_option = is_option_type(&field.ty);
        let json_type = rust_type_to_json_type(&field.ty);

        // A field is required if explicitly marked required, or if it's not Option
        // and not explicitly marked not required.
        let required = if is_option {
            param_required.unwrap_or(false)
        } else {
            param_required.unwrap_or(true)
        };

        let description = param_description.unwrap_or_default();

        if required {
            required_names.push(field_name_str.clone());
        }

        field_meta.push((field_name_str, description, json_type, required));
    }

    // Build the property entries for serde_json::json! macro.
    // Each entry is a token stream like: "field_name": { "type": "...", "description": "..." }
    let property_entries: Vec<proc_macro2::TokenStream> = field_meta
        .iter()
        .map(|(name, desc, ty, _)| {
            quote! {
                #name: {
                    "type": #ty,
                    "description": #desc
                }
            }
        })
        .collect();

    let required_entries: Vec<proc_macro2::TokenStream> = required_names
        .iter()
        .map(|name| quote! { #name })
        .collect();

    let expanded = quote! {
        impl #struct_name {
            /// The MCP tool name.
            pub const TOOL_NAME: &'static str = #tool_name;

            /// The MCP tool description.
            pub const TOOL_DESCRIPTION: &'static str = #tool_description;

            /// Returns the JSON Schema for this tool's input parameters.
            pub fn json_schema() -> serde_json::Value {
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        #(#property_entries),*
                    },
                    "required": [
                        #(#required_entries),*
                    ]
                })
            }

            /// Returns the full MCP tool definition including name, description, and schema.
            pub fn definition() -> serde_json::Value {
                serde_json::json!({
                    "name": Self::TOOL_NAME,
                    "description": Self::TOOL_DESCRIPTION,
                    "inputSchema": Self::json_schema()
                })
            }
        }
    };

    Ok(expanded)
}

/// Extract tool name and description from `#[tool(name = "...", description = "...")]`.
fn extract_tool_attrs(input: &DeriveInput) -> syn::Result<(String, String)> {
    let mut name = None;
    let mut description = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("tool") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;
                if let Lit::Str(s) = lit {
                    name = Some(s.value());
                } else {
                    return Err(meta.error("expected string literal for `name`"));
                }
            } else if meta.path.is_ident("description") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;
                if let Lit::Str(s) = lit {
                    description = Some(s.value());
                } else {
                    return Err(meta.error("expected string literal for `description`"));
                }
            } else {
                return Err(meta.error("unknown tool attribute, expected `name` or `description`"));
            }
            Ok(())
        })?;
    }

    let name = name.ok_or_else(|| {
        syn::Error::new_spanned(input, "ToolDef requires `#[tool(name = \"...\")]` attribute")
    })?;
    let description = description.unwrap_or_default();

    Ok((name, description))
}

/// Extract parameter description and required flag from `#[tool_param(...)]`.
fn extract_tool_param_attrs(field: &syn::Field) -> syn::Result<(Option<String>, Option<bool>)> {
    let mut description = None;
    let mut required = None;

    for attr in &field.attrs {
        if !attr.path().is_ident("tool_param") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("description") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;
                if let Lit::Str(s) = lit {
                    description = Some(s.value());
                } else {
                    return Err(meta.error("expected string literal for `description`"));
                }
            } else if meta.path.is_ident("required") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;
                if let Lit::Bool(b) = lit {
                    required = Some(b.value());
                } else {
                    return Err(meta.error("expected boolean literal for `required`"));
                }
            } else {
                return Err(meta.error("unknown tool_param attribute, expected `description` or `required`"));
            }
            Ok(())
        })?;
    }

    Ok((description, required))
}

/// Check if a type is `Option<T>`.
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

/// Map a Rust type to its JSON Schema type string.
fn rust_type_to_json_type(ty: &syn::Type) -> &'static str {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = segment.ident.to_string();
            match ident.as_str() {
                "String" | "str" => "string",
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize" => {
                    "integer"
                }
                "f32" | "f64" => "number",
                "bool" => "boolean",
                "Vec" | "VecDeque" | "BTreeSet" | "HashSet" | "LinkedList" => "array",
                "Option" => {
                    // Extract the inner type and recurse.
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                    {
                        return rust_type_to_json_type(inner);
                    }
                    "string" // Fallback.
                }
                _ => "string", // Default to string for unknown types.
            }
        } else {
            "string"
        }
    } else {
        "string"
    }
}

/// Derive macro that generates WASM↔Core type bridging code.
///
/// Converts between native Rust types and their WASM-compatible representations,
/// handling serialization boundaries and memory ownership transfer.
///
/// # Generated Items
///
/// - An `impl WasmBridge for YourType` block providing JSON serialization helpers
///
/// # Example
///
/// ```ignore
/// #[derive(WasmBridge, Serialize, Deserialize)]
/// #[bridge(target = "wasm")]
/// pub struct ToolCall {
///     pub name: String,
///     pub arguments: serde_json::Value,
/// }
/// ```
///
/// The `WasmBridge` trait must be in scope where this macro is used.
/// In production, it is defined in `antikythera-core`.
#[proc_macro_derive(WasmBridge, attributes(bridge))]
pub fn derive_wasm_bridge(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let type_name_str = name.to_string();

    let target = parse_bridge_target(&input.attrs);

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

    TokenStream::from(expanded)
}

/// Extract the `target` value from `#[bridge(target = "...")]` attribute.
/// Defaults to `"wasm"` if the attribute is absent or malformed.
fn parse_bridge_target(attrs: &[syn::Attribute]) -> String {
    for attr in attrs {
        if !attr.path().is_ident("bridge") {
            continue;
        }
        let mut target = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("target") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                target = Some(lit.value());
            }
            Ok(())
        });
        if let Some(t) = target {
            return t;
        }
    }
    "wasm".to_string()
}

/// Derive macro that generates JSON Schema from struct definitions.
///
/// Produces a `json_schema()` method returning the schema as a `serde_json::Value`,
/// useful for runtime type validation and tool parameter documentation.
///
/// # Requirements
///
/// The struct must have named fields. The generated code requires `serde_json` in scope.
///
/// # Rules
///
/// - Fields without `#[serde(default)]` and without `Option<T>` are marked as required.
/// - Primitive types map to their JSON Schema equivalents.
/// - `Vec<T>` maps to `{"type": "array", "items": <schema of T>}`.
/// - `Option<T>` maps to the inner type's schema and is never required.
/// - Nested structs generate `$ref` entries with placeholder definitions.
///
/// # Example
///
/// ```ignore
/// #[derive(JsonSchema)]
/// struct Config {
///     name: String,
///     #[serde(default)]
///     enabled: bool,
/// }
///
/// // Config::json_schema() returns:
/// // {
/// //   "type": "object",
/// //   "properties": {
/// //     "name": {"type": "string"},
/// //     "enabled": {"type": "boolean"}
/// //   },
/// //   "required": ["name"],
/// //   "definitions": {}
/// // }
/// ```
#[proc_macro_derive(JsonSchema, attributes(serde))]
pub fn derive_json_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_json_schema(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn impl_json_schema(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "JsonSchema can only be derived on structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "JsonSchema can only be derived on structs",
            ));
        }
    };

    // Collect property insertions, required field names, and definition insertions.
    let mut prop_inserts: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut required_names: Vec<String> = Vec::new();
    // (type_name, definition_schema_expr)
    let mut def_inserts: Vec<(String, proc_macro2::TokenStream)> = Vec::new();

    for field in fields {
        let field_name_str = field.ident.as_ref().unwrap().to_string();

        let has_default = has_serde_default(&field.attrs);
        let is_option = is_option_type(&field.ty);

        let required = !has_default && !is_option;
        if required {
            required_names.push(field_name_str.clone());
        }

        let (schema_expr, nested_defs) = field_type_schema(&field.ty)?;
        prop_inserts.push(quote! {
            properties.insert(#field_name_str.to_string(), #schema_expr);
        });
        for (name, def_schema) in nested_defs {
            def_inserts.push((name, def_schema));
        }
    }

    // Deduplicate definitions: keep only the first occurrence of each type name.
    let mut seen_defs = std::collections::HashSet::new();
    let unique_def_inserts: Vec<proc_macro2::TokenStream> = def_inserts
        .into_iter()
        .filter_map(|(name, schema)| {
            if seen_defs.insert(name.clone()) {
                Some(quote! {
                    definitions.insert(#name.to_string(), #schema);
                })
            } else {
                None
            }
        })
        .collect();

    let required_tokens: Vec<proc_macro2::TokenStream> = required_names
        .iter()
        .map(|n| quote! { #n.to_string() })
        .collect();

    let expanded = quote! {
        impl #struct_name {
            /// Returns the JSON Schema for this type (draft-07).
            pub fn json_schema() -> serde_json::Value {
                let mut properties = serde_json::Map::new();
                #(#prop_inserts)*

                let mut definitions = serde_json::Map::new();
                #(#unique_def_inserts)*

                let required: Vec<String> = vec![#(#required_tokens),*];

                let mut schema = serde_json::Map::new();
                schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));
                schema.insert("properties".to_string(), serde_json::Value::Object(properties));
                if !required.is_empty() {
                    schema.insert(
                        "required".to_string(),
                        serde_json::to_value(&required).unwrap(),
                    );
                }
                schema.insert("definitions".to_string(), serde_json::Value::Object(definitions));

                serde_json::Value::Object(schema)
            }
        }
    };

    Ok(expanded)
}

/// Check if a field has the `#[serde(default)]` attribute.
fn has_serde_default(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        // Parse #[serde(default)] or #[serde(default = "...")].
        let mut found_default = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                found_default = true;
            }
            Ok(())
        });
        if found_default {
            return true;
        }
    }
    false
}

/// Generate JSON Schema expression for a field type.
/// Returns (schema_expr, nested_type_definitions).
/// Each nested definition is (type_name, definition_schema_expr).
fn field_type_schema(
    ty: &syn::Type,
) -> syn::Result<(proc_macro2::TokenStream, Vec<(String, proc_macro2::TokenStream)>)> {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let ident = segment.ident.to_string();
        match ident.as_str() {
            "String" | "str" => {
                return Ok((quote! { serde_json::json!({"type": "string"}) }, Vec::new()));
            }
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize" => {
                return Ok((quote! { serde_json::json!({"type": "integer"}) }, Vec::new()));
            }
            "f32" | "f64" => {
                return Ok((quote! { serde_json::json!({"type": "number"}) }, Vec::new()));
            }
            "bool" => {
                return Ok((quote! { serde_json::json!({"type": "boolean"}) }, Vec::new()));
            }
            "Vec" | "VecDeque" | "BTreeSet" | "HashSet" | "LinkedList" => {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                    && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                {
                    let (items_schema, nested) = field_type_schema(inner)?;
                    return Ok((
                        quote! { serde_json::json!({"type": "array", "items": #items_schema}) },
                        nested,
                    ));
                }
                return Ok((quote! { serde_json::json!({"type": "array"}) }, Vec::new()));
            }
            "Option" => {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                    && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                {
                    return field_type_schema(inner);
                }
                return Ok((quote! { serde_json::json!({"type": "string"}) }, Vec::new()));
            }
            _ => {
                // Nested struct — generate $ref and a placeholder definition.
                let type_name = ident.clone();
                let ref_str = format!("#/definitions/{}", type_name);
                let schema = quote! { serde_json::json!({"$ref": #ref_str}) };
                let definition = quote! { serde_json::json!({"type": "object"}) };
                return Ok((schema, vec![(type_name, definition)]));
            }
        }
    }
    // Fallback for non-path types (tuples, references, etc).
    Ok((quote! { serde_json::json!({"type": "string"}) }, Vec::new()))
}

/// Derive macro that validates FSM transition completeness at compile time.
///
/// Ensures every state has a defined transition to all other required states,
/// preventing runtime errors from missing state machine paths.
///
/// # Example
///
/// ```ignore
/// #[derive(FsmComplete)]
/// #[fsm_transitions(
///     Idle => [UserTurnPrepared],
///     UserTurnPrepared => [LlmStreaming],
///     LlmStreaming => [LlmCommitted],
///     LlmCommitted => [ToolRequested, Final, Idle],
///     ToolRequested => [ToolResultProcessed],
///     ToolResultProcessed => [LlmStreaming, Final, Idle],
///     Final => [Idle]
/// )]
/// pub enum AgentFsmState {
///     Idle,
///     UserTurnPrepared,
///     LlmStreaming,
///     LlmCommitted,
///     ToolRequested,
///     ToolResultProcessed,
///     Final,
/// }
/// ```
#[proc_macro_derive(FsmComplete, attributes(fsm_transitions))]
pub fn derive_fsm_complete(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_fsm_complete(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn impl_fsm_complete(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let enum_name = &input.ident;

    // Only enums are supported.
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "FsmComplete can only be derived on enums",
            ));
        }
    };

    // Collect enum variant names.
    let enum_variants: Vec<String> = variants.iter().map(|v| v.ident.to_string()).collect();

    // Parse the #[fsm_transitions(...)] attribute.
    let transition_matrix = parse_fsm_transitions(input)?;

    // Validate: all states in the matrix exist in the enum.
    for source_state in transition_matrix.keys() {
        if !enum_variants.contains(source_state) {
            return Err(syn::Error::new_spanned(
                input,
                format!(
                    "FSM transition matrix references unknown state '{}'",
                    source_state
                ),
            ));
        }
        for target_state in transition_matrix.get(source_state).unwrap() {
            if !enum_variants.contains(target_state) {
                return Err(syn::Error::new_spanned(
                    input,
                    format!(
                        "FSM transition matrix references unknown state '{}'",
                        target_state
                    ),
                ));
            }
        }
    }

    // Validate: all enum variants appear in the transition matrix.
    for variant in &enum_variants {
        if !transition_matrix.contains_key(variant) {
            return Err(syn::Error::new_spanned(
                input,
                format!(
                    "FSM state '{}' has no outgoing transitions defined",
                    variant
                ),
            ));
        }
    }

    // Validate: every state has at least one outgoing transition.
    for (source, targets) in &transition_matrix {
        if targets.is_empty() {
            return Err(syn::Error::new_spanned(
                input,
                format!("FSM state '{}' has no outgoing transitions", source),
            ));
        }
    }

    // Generate a const block with compile-time assertions.
    let assertions = generate_fsm_assertions(enum_name, &enum_variants, &transition_matrix);

    let expanded = quote! {
        const _: () = {
            #assertions
        };
    };

    Ok(expanded)
}

/// A single transition: source state => list of target states.
struct FsmTransition {
    source: syn::Ident,
    targets: Vec<syn::Ident>,
}

impl syn::parse::Parse for FsmTransition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let source: syn::Ident = input.parse()?;
        let _arrow: syn::Token![=>] = input.parse()?;
        let content;
        syn::bracketed!(content in input);
        let targets = content.parse_terminated(syn::Ident::parse, syn::Token![,])?;
        Ok(FsmTransition {
            source,
            targets: targets.into_iter().collect(),
        })
    }
}

struct FsmTransitionsAttr {
    transitions: Vec<FsmTransition>,
}

impl syn::parse::Parse for FsmTransitionsAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut transitions = Vec::new();
        while !input.is_empty() {
            transitions.push(input.parse::<FsmTransition>()?);
            // Optionally consume a trailing comma.
            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }
        Ok(FsmTransitionsAttr { transitions })
    }
}

/// Parse the `#[fsm_transitions(...)]` attribute and return a transition matrix.
fn parse_fsm_transitions(
    input: &DeriveInput,
) -> syn::Result<std::collections::HashMap<String, Vec<String>>> {
    let mut attr_iter = input.attrs.iter().filter(|a| a.path().is_ident("fsm_transitions"));

    let attr = attr_iter.next().ok_or_else(|| {
        syn::Error::new_spanned(
            input,
            "FsmComplete requires #[fsm_transitions(...)] attribute",
        )
    })?;

    if attr_iter.next().is_some() {
        return Err(syn::Error::new_spanned(
            input,
            "FsmComplete allows only one #[fsm_transitions(...)] attribute",
        ));
    }

    let parsed: FsmTransitionsAttr = attr.parse_args()?;

    let mut matrix = std::collections::HashMap::new();
    for transition in &parsed.transitions {
        let source = transition.source.to_string();
        let targets: Vec<String> = transition.targets.iter().map(|t| t.to_string()).collect();
        matrix.insert(source, targets);
    }

    if matrix.is_empty() {
        return Err(syn::Error::new_spanned(
            input,
            "FsmComplete requires at least one transition in #[fsm_transitions(...)]",
        ));
    }

    Ok(matrix)
}

/// Generate compile-time assertions for FSM validation.
fn generate_fsm_assertions(
    enum_name: &syn::Ident,
    enum_variants: &[String],
    transition_matrix: &std::collections::HashMap<String, Vec<String>>,
) -> proc_macro2::TokenStream {
    // Build a set of all states that are targets of transitions (reachable states).
    let mut reachable = std::collections::HashSet::new();
    for targets in transition_matrix.values() {
        for target in targets {
            reachable.insert(target.clone());
        }
    }

    // Generate error messages for unreachable states (except the first state which is the initial state).
    let mut unreachable_errors = Vec::new();
    if let Some(first_variant) = enum_variants.first() {
        for variant in enum_variants {
            if variant != first_variant && !reachable.contains(variant) {
                let msg = format!(
                    "FSM state '{}' is not reachable from any other state",
                    variant
                );
                unreachable_errors.push(quote! {
                    compile_error!(#msg);
                });
            }
        }
    }

    // Generate error messages for states with no outgoing transitions.
    let mut no_outgoing_errors = Vec::new();
    for variant in enum_variants {
        if !transition_matrix.contains_key(variant) {
            let msg = format!(
                "FSM state '{}' has no outgoing transitions defined",
                variant
            );
            no_outgoing_errors.push(quote! {
                compile_error!(#msg);
            });
        } else if transition_matrix.get(variant).unwrap().is_empty() {
            let msg = format!("FSM state '{}' has no outgoing transitions", variant);
            no_outgoing_errors.push(quote! {
                compile_error!(#msg);
            });
        }
    }

    // Generate error messages for states in matrix but not in enum.
    let mut unknown_state_errors = Vec::new();
    for source in transition_matrix.keys() {
        if !enum_variants.contains(source) {
            let msg = format!(
                "FSM transition matrix references unknown state '{}'",
                source
            );
            unknown_state_errors.push(quote! {
                compile_error!(#msg);
            });
        }
    }
    for targets in transition_matrix.values() {
        for target in targets {
            if !enum_variants.contains(target) {
                let msg = format!(
                    "FSM transition matrix references unknown state '{}'",
                    target
                );
                unknown_state_errors.push(quote! {
                    compile_error!(#msg);
                });
            }
        }
    }

    quote! {
        #( #unreachable_errors )*
        #( #no_outgoing_errors )*
        #( #unknown_state_errors )*

        // Verify that the enum type exists (ensures the macro is used on a valid type).
        let _: #enum_name;
    }
}

/// Derive macro that generates compile-time validation for port trait implementations.
///
/// Generates a compile-time assertion that the annotated struct implements
/// the specified port trait. If any required method is missing, the compiler
/// will produce an error at the derive site.
///
/// # Example
///
/// ```ignore
/// #[derive(PortValidate)]
/// #[implements(ModelClient)]
/// pub struct OllamaClient { ... }
/// ```
///
/// This generates:
///
/// ```ignore
/// const _: () = {
///     fn _assert_port<T: ?Sized + ModelClient>() {}
///     fn _assert_impl() {
///         _assert_port::<OllamaClient>();
///     }
/// };
/// ```
#[proc_macro_derive(PortValidate, attributes(implements))]
pub fn derive_port_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_port_validate(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn impl_port_validate(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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

