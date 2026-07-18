// Provider config tests - testing ModelProviderConfig behavior
//
// Tests for provider type detection and helper methods.
// Uses CLI's ModelProviderConfig directly — no file I/O required.

use antikythera_cli::infrastructure::llm::ModelProviderConfig;
use antikythera_core::domain::content::{
    ContentItem, FileContent, FileMetadata, parse_step_output,
};
use antikythera_core::domain::types::{ChatMessage, MessagePart, MessageRole};
use antikythera_core::infrastructure::model::{HostModelResponse, ModelError};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde_json::json;

// Split into 8 parts for consistent test organization.
include!("type_detection_tests/empty_placeholder.rs");
include!("type_detection_tests/ollama_case_insensitive.rs");
include!("type_detection_tests/empty_placeholder.rs");
include!("type_detection_tests/gemini_case_insensitive.rs");
include!("type_detection_tests/mixed_case_provider.rs");
include!("type_detection_tests/host_model_response.rs");
include!("type_detection_tests/parse_step_output.rs");
include!("type_detection_tests/file_content_decode.rs");
