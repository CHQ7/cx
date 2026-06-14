use async_trait::async_trait;
use futures::stream::BoxStream;

use super::client::{LlmClient, LlmError, ToolSchema};
use super::models::{Message, MockResponse};

/// MixinSession tries multiple LLM clients in order, falling back on failure
pub struct MixinSession {
    sessions: Vec<Box<dyn LlmClient>>,
}

impl MixinSession {
    pub fn new(sessions: Vec<Box<dyn LlmClient>>) -> Self {
        Self { sessions }
    }
}

#[async_trait]
impl LlmClient for MixinSession {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSchema>>,
    ) -> Result<MockResponse, LlmError> {
        let mut last_error = None;

        for session in &self.sessions {
            match session.chat(messages.clone(), tools.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or(LlmError::ApiError("All sessions failed".to_string())))
    }

    async fn chat_stream(
        &self,
        _messages: Vec<Message>,
        _tools: Option<Vec<ToolSchema>>,
    ) -> Result<BoxStream<'static, Result<String, LlmError>>, LlmError> {
        todo!("stream not yet implemented")
    }
}
