use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::models::{Message, MockResponse};

// LlmError - all possible LLM errors
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Rate limited")]
    RateLimited,
}

// ToolSchema - describes a tool for the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

// LlmClient trait - all LLM providers implement this
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send messages and get a complete response
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSchema>>,
    ) -> Result<MockResponse, LlmError>;

    /// Send messages and get a streaming response (SSE)
    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSchema>>,
    ) -> Result<BoxStream<'static, Result<String, LlmError>>, LlmError>;
}
