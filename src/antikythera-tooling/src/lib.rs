mod envelope;
pub mod error;
pub mod interface;
pub mod manager;

pub mod transport;

pub use envelope::{
    EnvelopeError, ToolCallEnvelope, ToolResultEnvelope, validate_tool_call_envelope,
    validate_tool_result_envelope,
};
pub use error::ToolInvokeError;
pub use interface::{
    PROTOCOL_VERSION, ServerToolInfo, TaskSupport, ToolAnnotations, ToolExecution, ToolIcon,
    ToolServerInterface,
};
pub use manager::ServerInstance;
pub use manager::ServerManager;
pub use manager::TransportFactory;
pub use transport::McpTransport;
