use antikythera_macros::ToolDef;
use serde::Serialize;

#[derive(ToolDef, Serialize)]
#[tool(name = "test_tool", description = "A test tool")]
struct TestTool {
    #[tool_param(description = "Input text")]
    input: String,
    #[tool_param(description = "Optional count", required = false)]
    count: Option<i32>,
}

#[test]
fn test_tool_name() {
    assert_eq!(TestTool::TOOL_NAME, "test_tool");
}

#[test]
fn test_tool_description() {
    assert_eq!(TestTool::TOOL_DESCRIPTION, "A test tool");
}

#[test]
fn test_tool_schema() {
    let schema = TestTool::json_schema();
    assert!(schema.is_object());
    assert_eq!(schema["properties"]["input"]["type"], "string");
    assert_eq!(schema["properties"]["input"]["description"], "Input text");
    assert_eq!(schema["properties"]["count"]["type"], "integer");
    assert_eq!(
        schema["properties"]["count"]["description"],
        "Optional count"
    );
    assert!(schema["required"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("input")));
    assert!(!schema["required"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("count")));
}

#[test]
fn test_tool_definition() {
    let def = TestTool::definition();
    assert_eq!(def["name"], "test_tool");
    assert_eq!(def["description"], "A test tool");
    assert!(def["inputSchema"].is_object());
}
