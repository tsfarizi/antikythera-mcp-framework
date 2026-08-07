//! Integration tests verifying `antikythera-macros` derives work across crate boundaries
//! when used within the `antikythera-ports` crate's test context.

#![allow(dead_code)]

use antikythera_macros::{PortValidate, ToolDef, JsonSchema};
use antikythera_ports::{AppLogger, LogProvider, LogQueryPort};
use antikythera_ports::types::{LogEntry, LogFilter};

#[derive(PortValidate)]
#[implements(AppLogger)]
struct ValidatedLogger;

impl AppLogger for ValidatedLogger {
    fn log_info(&self, _message: String) {}
    fn log_warn(&self, _message: String) {}
    fn log_error(&self, _message: String) {}
    fn log_debug(&self, _message: String) {}
}

#[test]
fn port_validate_compiles_with_app_logger() {
    // If this compiles, PortValidate confirmed ValidatedLogger implements AppLogger.
    let logger = ValidatedLogger;
    logger.log_info("test".to_string());
}

// ============================================================================
// ToolDef: verify macro works in ports crate context
// ============================================================================

#[derive(ToolDef, serde::Serialize)]
#[tool(name = "ports_test_tool", description = "Tool defined in ports integration test")]
struct PortsTestTool {
    #[tool_param(description = "Query string")]
    query: String,
    #[tool_param(description = "Max results", required = false)]
    max_results: Option<u32>,
}

#[test]
fn tool_def_cross_crate_constants() {
    assert_eq!(PortsTestTool::TOOL_NAME, "ports_test_tool");
    assert_eq!(
        PortsTestTool::TOOL_DESCRIPTION,
        "Tool defined in ports integration test"
    );
}

#[test]
fn tool_def_cross_crate_schema() {
    let schema = PortsTestTool::json_schema();
    assert!(schema.is_object());
    assert_eq!(schema["properties"]["query"]["type"], "string");
    assert_eq!(schema["properties"]["max_results"]["type"], "integer");

    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::json!("query")));
    assert!(!required.contains(&serde_json::json!("max_results")));
}

#[test]
fn tool_def_cross_crate_definition() {
    let def = PortsTestTool::definition();
    assert_eq!(def["name"], "ports_test_tool");
    assert!(def["inputSchema"].is_object());
}

// ============================================================================
// JsonSchema: verify macro works in ports crate context
// ============================================================================

#[derive(JsonSchema, serde::Serialize)]
struct PortsConfig {
    host: String,
    port: u16,
    #[serde(default)]
    debug: bool,
}

#[test]
fn json_schema_cross_crate() {
    let schema = PortsConfig::json_schema();
    assert!(schema.is_object());
    assert_eq!(schema["properties"]["host"]["type"], "string");
    assert_eq!(schema["properties"]["port"]["type"], "integer");
    assert_eq!(schema["properties"]["debug"]["type"], "boolean");

    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&serde_json::json!("host")));
    assert!(required.contains(&serde_json::json!("port")));
    assert!(!required.contains(&serde_json::json!("debug")));
}

// ============================================================================
// Trait accessibility: verify port traits are importable and well-formed
// ============================================================================

#[test]
fn port_traits_are_accessible() {
    // Compile-time checks that port traits are importable and have expected shape.
    fn _assert_app_logger<T: AppLogger>() {}
    fn _assert_log_provider<T: LogProvider>() {}
    fn _assert_log_query<T: LogQueryPort>() {}
}

#[test]
fn port_types_use_domain_types() {
    // Verify that ports crate types depend on domain types correctly.
    let entry = LogEntry::new(
        antikythera_ports::types::LogLevel::Info,
        "integration test",
    );
    assert_eq!(entry.level, antikythera_ports::types::LogLevel::Info);
    assert_eq!(entry.message, "integration test");

    let filter = LogFilter::new()
        .min_level(antikythera_ports::types::LogLevel::Warn)
        .limit(10);
    assert!(!filter.matches(&entry)); // Info < Warn
}
