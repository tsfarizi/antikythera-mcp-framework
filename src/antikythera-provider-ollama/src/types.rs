use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub message: Message,
    pub done: bool,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListModelsResponse {
    pub models: Vec<ModelInfo>,
}
