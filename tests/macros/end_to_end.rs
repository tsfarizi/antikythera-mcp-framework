//! End-to-end tests for antikythera-macros derive macros.
//!
//! These tests verify that the macros work correctly when used
//! from outside the macro crate (cross-crate boundary).

#![allow(dead_code)]

use antikythera_macros::{ToolDef, WasmBridge, JsonSchema, FsmComplete, PortValidate};
use serde::{Serialize, Deserialize};

// ============================================================================
// WasmBridge trait: must be in scope for the macro-generated impl
// ============================================================================

/// Marker trait for types that have WASM bridge support.
/// The macro generates `impl WasmBridge for T` — this trait must be importable.
pub trait WasmBridge: Sized {
    const WASM_TYPE_NAME: &'static str;
    const BRIDGE_TARGET: &'static str;
    fn to_json_value(&self) -> Result<serde_json::Value, String>;
    fn from_json_value(value: serde_json::Value) -> Result<Self, String>;
}

// ============================================================================
// ToolDef Test
// ============================================================================

#[derive(ToolDef, Serialize)]
#[tool(name = "e2e_test_tool", description = "End-to-end test tool")]
struct E2eTestTool {
    #[tool_param(description = "Input text")]
    input: String,
    #[tool_param(description = "Optional count", required = false)]
    count: Option<i32>,
}

#[test]
fn e2e_tool_def_constants() {
    assert_eq!(E2eTestTool::TOOL_NAME, "e2e_test_tool");
    assert_eq!(E2eTestTool::TOOL_DESCRIPTION, "End-to-end test tool");
}

#[test]
fn e2e_tool_def_schema() {
    let schema = E2eTestTool::json_schema();
    assert!(schema.is_object());
    assert_eq!(schema["properties"]["input"]["type"], "string");
    assert_eq!(schema["properties"]["input"]["description"], "Input text");
    assert_eq!(schema["properties"]["count"]["type"], "integer");
    assert_eq!(schema["properties"]["count"]["description"], "Optional count");

    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::json!("input")));
    assert!(!required.contains(&serde_json::json!("count")));
}

#[test]
fn e2e_tool_def_definition() {
    let def = E2eTestTool::definition();
    assert_eq!(def["name"], "e2e_test_tool");
    assert_eq!(def["description"], "End-to-end test tool");
    assert!(def["inputSchema"].is_object());
}

// ============================================================================
// WasmBridge Test
// ============================================================================

#[derive(WasmBridge, Serialize, Deserialize)]
struct E2eBridge {
    name: String,
    value: i32,
}

#[test]
fn e2e_wasm_bridge_type_name() {
    assert_eq!(
        <E2eBridge as WasmBridge>::WASM_TYPE_NAME,
        "E2eBridge"
    );
}

#[test]
fn e2e_wasm_bridge_target() {
    assert_eq!(<E2eBridge as WasmBridge>::BRIDGE_TARGET, "wasm");
}

#[test]
fn e2e_wasm_bridge_roundtrip() {
    let item = E2eBridge {
        name: "test".to_string(),
        value: 42,
    };
    let json = item.to_json_value().unwrap();
    assert_eq!(json["name"], "test");
    assert_eq!(json["value"], 42);

    let restored = E2eBridge::from_json_value(json).unwrap();
    assert_eq!(restored.name, "test");
    assert_eq!(restored.value, 42);
}

#[test]
fn e2e_wasm_bridge_invalid_json() {
    let json = serde_json::json!({"invalid": true});
    let result = E2eBridge::from_json_value(json);
    assert!(result.is_err());
}

// ============================================================================
// JsonSchema Test
// ============================================================================

#[derive(JsonSchema, Serialize)]
struct E2eConfig {
    name: String,
    #[serde(default)]
    optional: Option<i32>,
}

#[test]
fn e2e_json_schema_structure() {
    let schema = E2eConfig::json_schema();
    assert!(schema.is_object());
    assert_eq!(schema["type"], "object");
}

#[test]
fn e2e_json_schema_properties() {
    let schema = E2eConfig::json_schema();
    let props = schema["properties"].as_object().unwrap();
    assert!(props.contains_key("name"));
    assert!(props.contains_key("optional"));
    assert_eq!(props["name"]["type"], "string");
    assert_eq!(props["optional"]["type"], "integer");
}

#[test]
fn e2e_json_schema_required() {
    let schema = E2eConfig::json_schema();
    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::json!("name")));
    assert!(!required.contains(&serde_json::json!("optional")));
}

// ============================================================================
// FsmComplete Test
// ============================================================================

#[derive(FsmComplete)]
#[fsm_transitions(
    A => [B],
    B => [C],
    C => [A]
)]
enum E2eFsm {
    A,
    B,
    C,
}

#[test]
fn e2e_fsm_compiles() {
    // If this compiles, FsmComplete validated the transition matrix at compile time.
    let _ = std::mem::size_of::<E2eFsm>();
}

#[test]
fn e2e_fsm_variants_exist() {
    let a = E2eFsm::A;
    let b = E2eFsm::B;
    let c = E2eFsm::C;
    assert!(matches!(a, E2eFsm::A));
    assert!(matches!(b, E2eFsm::B));
    assert!(matches!(c, E2eFsm::C));
}

// ============================================================================
// PortValidate Test
// ============================================================================

trait E2ePort: Send + Sync {
    fn execute(&self, input: &str) -> String;
}

#[derive(PortValidate)]
#[implements(E2ePort)]
struct E2ePortImpl {
    label: String,
}

impl E2ePort for E2ePortImpl {
    fn execute(&self, input: &str) -> String {
        format!("{}: {}", self.label, input)
    }
}

#[test]
fn e2e_port_validate_compiles() {
    // If this compiles, PortValidate confirmed E2ePortImpl implements E2ePort.
    let impl_ = E2ePortImpl {
        label: "test".to_string(),
    };
    assert_eq!(impl_.execute("hello"), "test: hello");
}

// ============================================================================
// Cross-macro interaction: multiple derives on the same type
// ============================================================================

#[derive(JsonSchema, Serialize, Deserialize)]
struct E2eMultiDerive {
    key: String,
    #[serde(default)]
    value: Option<u64>,
}

#[test]
fn e2e_multi_derive_json_schema() {
    let schema = E2eMultiDerive::json_schema();
    assert!(schema.is_object());
    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::json!("key")));
    assert!(!required.contains(&serde_json::json!("value")));
}

#[test]
fn e2e_multi_derive_serialization() {
    let item = E2eMultiDerive {
        key: "test".to_string(),
        value: Some(42),
    };
    let json = serde_json::to_value(&item).unwrap();
    assert_eq!(json["key"], "test");
    assert_eq!(json["value"], 42);

    let restored: E2eMultiDerive = serde_json::from_value(json).unwrap();
    assert_eq!(restored.key, "test");
    assert_eq!(restored.value, Some(42));
}
