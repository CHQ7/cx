use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

use super::client::{LlmClient, LlmError, ToolSchema};
use super::models::{Message, MockFunction, MockResponse, MockToolCall};

pub struct OpenAiSession {
    api_key: String,
    api_base: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAiSession {
    pub fn new(api_key: String, api_base: String, model: String) -> Self {
        Self { api_key, api_base, model, client: reqwest::Client::new() }
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let auth = format!("Bearer {}", self.api_key);
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth).unwrap());
        headers
    }
}

#[async_trait]
impl LlmClient for OpenAiSession {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSchema>>,
    ) -> Result<MockResponse, LlmError> {
        let url = format!("{}/chat/completions", self.api_base);

        let mut body = json!({
            "model": self.model,
            "messages": messages,
        });

        if let Some(tools) = tools {
            body["tools"] = json!(tools);
        }

        let resp = self.client
            .post(&url)
            .headers(self.build_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::NetworkError(e.to_string()))?;

        if resp.status().is_success() {
            let json: serde_json::Value = resp.json().await
                .map_err(|e| LlmError::ParseError(e.to_string()))?;

            let choice = &json["choices"][0];
            let message = &choice["message"];

            let content = message["content"].as_str().unwrap_or("").to_string();

            let tool_calls: Vec<MockToolCall> = message["tool_calls"].as_array()
                .map(|arr| arr.iter().map(|tc| MockToolCall {
                    function: MockFunction {
                        name: tc["function"]["name"].as_str().unwrap_or("").to_string(),
                        arguments: tc["function"]["arguments"].as_str().unwrap_or("{}").to_string(),
                    },
                    id: tc["id"].as_str().unwrap_or("").to_string(),
                }).collect())
                .unwrap_or_default();

            Ok(MockResponse {
                thinking: String::new(),
                content,
                tool_calls,
                raw: json.to_string(),
                stop_reason: choice["finish_reason"].as_str().unwrap_or("").to_string(),
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
        todo!("stream not yet implemented")
    }
}
