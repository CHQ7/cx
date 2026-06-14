use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;

use super::client::{LlmClient, LlmError, ToolSchema};
use super::models::Message;
use super::models::MockResponse;
use super::models::MockFunction;
use super::models::MockToolCall;

pub struct ClaudeSession {
    api_key: String,
    api_base: String,
    model: String,
    client: reqwest::Client,
}

impl ClaudeSession {
    pub fn new(api_key: String, api_base: String, model: String) -> Self {
        Self {
            api_key,
            api_base,
            model,
            client: reqwest::Client::new(),
        }
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("x-api-key", HeaderValue::from_str(&self.api_key).unwrap());
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        headers
    }
}

#[async_trait]
impl LlmClient for ClaudeSession {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSchema>>,
    ) -> Result<MockResponse, LlmError> {
        let url = format!("{}/v1/messages", self.api_base);

        let mut body = json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": messages,
        });

        if let Some(tools) = tools {
            body["tools"] = json!(tools);
        }

        let resp = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        if resp.status().is_success() {
            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| LlmError::ParseError(e.to_string()))?;

            // Parse content blocks
            let content_blocks = json["content"]
                .as_array()
                .map(|arr| arr.to_vec())
                .unwrap_or_default();

            let mut content_text = String::new();
            let mut tool_calls = Vec::new();

            for block in content_blocks {
                match block.get("type").and_then(|v| v.as_str()) {
                    Some("text") => {
                        if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                            content_text.push_str(text);
                        }
                    }
                    Some("tool_use") => {
                        let name = block
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let id = block
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let input = block.get("input").cloned().unwrap_or(json!({}));

                        tool_calls.push(MockToolCall {
                            function: MockFunction {
                                name,
                                arguments: input.to_string(),
                            },
                            id,
                        });
                    }
                    _ => {}
                }
            }

            Ok(MockResponse {
                thinking: String::new(),
                content: content_text,
                tool_calls,
                raw: json.to_string(),
                stop_reason: json["stop_reason"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            })
        } else {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            Err(LlmError::ApiError(format!("HTTP {}: {}", status, text)))
        }
    }

    async fn chat_stream(
        &self,
        _messages: Vec<Message>,
        _tools: Option<Vec<ToolSchema>>,
    ) -> Result<futures::stream::BoxStream<'static, Result<String, LlmError>>, LlmError> {
        // TODO: Implement SSE streaming in a later task
        todo!("stream not yet implemented")
    }
}
