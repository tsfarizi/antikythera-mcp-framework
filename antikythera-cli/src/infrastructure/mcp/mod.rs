//! MCP protocol contracts (CLI-owned).
pub mod contract;

pub use contract::{
    ContractValidator, ToolCallEnvelope, ToolExecutionError, ToolResultEnvelope, validate_tool_name,
};
