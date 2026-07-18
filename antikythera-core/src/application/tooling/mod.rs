mod envelope;
pub(crate) mod error;
pub(crate) mod interface;
pub(crate) mod manager;

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
pub use manager::ServerManager;
pub use transport::McpTransport;

// Re-export infrastructure types for backward compatibility.
pub use crate::infrastructure::transport::{
    BuiltinTransport, HttpTransport, HttpTransportConfig, TransportMode,
};
pub use crate::infrastructure::transport::TransportFactory;
#[cfg(feature = "native-transport")]
pub use crate::infrastructure::transport::spawn_and_list_tools;
