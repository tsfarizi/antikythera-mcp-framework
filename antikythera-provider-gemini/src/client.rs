use antikythera_core::domain::types::{ChatMessage, MessageRole};
use antikythera_core::infrastructure::model::{ModelError, ModelRequest, ModelResponse};
use antikythera_core::infrastructure::model::traits::ModelClient;
use crate::types::*;

pub struct GeminiClient {
    id: String,
    api_key: String,
    model: String,
    endpoint: String,
    client: reqwest::Client,
}

impl GeminiClient {
    pub fn new(api_key: &str, model: &str) -> Result<Self, ModelError> {
        if api_key.is_empty() {
            return Err(ModelError::missing_api_key("gemini"));
        }
        let id = format!("gemini-{}", model);
        Ok(Self {
            id,
            api_key: api_key.to_string(),
            model: model.to_string(),
            endpoint: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            client: reqwest::Client::new(),
        })
    }

    pub fn with_endpoint(endpoint: &str, api_key: &str, model: &str) -> Result<Self, ModelError> {
        if api_key.is_empty() {
            return Err(ModelError::missing_api_key("gemini"));
        }
        let id = format!("gemini-{}", model);
        Ok(Self {
            id,
            api_key: api_key.to_string(),
            model: model.to_string(),
            endpoint: endpoint.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        })
    }

    fn provider_name(&self) -> &str {
        "gemini"
    }

    fn map_error(&self, e: reqwest::Error) -> ModelError {
        ModelError::network(self.provider_name(), e.to_string())
    }
}

#[async_trait::async_trait]
impl ModelClient for GeminiClient {
    fn id(&self) -> &str {
        &self.id
    }

    async fn chat(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let mut system_instruction = None;
        let mut contents = Vec::new();

        for msg in &request.messages {
            match msg.role {
                MessageRole::System => {
                    system_instruction = Some(Content {
                        parts: vec![Part {
                            text: msg.content(),
                        }],
                        role: None,
                    });
                }
                MessageRole::User => {
                    contents.push(Content {
                        parts: vec![Part {
                            text: msg.content(),
                        }],
                        role: Some("user".to_string()),
                    });
                }
                MessageRole::Assistant => {
                    contents.push(Content {
                        parts: vec![Part {
                            text: msg.content(),
                        }],
                        role: Some("model".to_string()),
                    });
                }
                MessageRole::ToolResult => {
                    contents.push(Content {
                        parts: vec![Part {
                            text: msg.content(),
                        }],
                        role: Some("user".to_string()),
                    });
                }
            }
        }

        let response_mime_type = request
            .params
            .get("output_format")
            .and_then(|v| v.as_str())
            .map(|_| "application/json".to_string());

        let generation_config = response_mime_type.map(|mime| GenerationConfig {
            response_mime_type: Some(mime),
        });

        let gemini_request = GeminiRequest {
            contents,
            system_instruction,
            generation_config,
        };

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.endpoint, self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&gemini_request)
            .send()
            .await
            .map_err(|e| self.map_error(e))?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            if let Ok(gemini_err) = serde_json::from_str::<GeminiError>(&body) {
                return Err(ModelError::network(
                    self.provider_name(),
                    gemini_err.error.message,
                ));
            }
            return Err(ModelError::network(
                self.provider_name(),
                format!("HTTP {status}: {body}"),
            ));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| ModelError::invalid_response(self.provider_name(), e.to_string()))?;

        let content = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_default();

        let tokens = gemini_response
            .usage_metadata
            .and_then(|u| u.total_token_count)
            .map(|t| t as u64)
            .unwrap_or(0);

        Ok(ModelResponse {
            message: ChatMessage::new(MessageRole::Assistant, content),
            session_id: request.session_id,
            tokens,
        })
    }
}
