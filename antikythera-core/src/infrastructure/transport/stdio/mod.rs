//! STDIO transport adapter for MCP server processes.

pub(crate) mod jsonrpc_client;
pub mod process;

pub(crate) use process::{McpProcess, McpProcessInner};
