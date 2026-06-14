use serde::{Deserialize, Serialize};

// Role enum - matches Python's role strings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
}

// ContentBlock - Claude content-block format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    Thinking { thinking: String, signature: Option<String> },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String },
}

// Message - a single message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_results: Option<Vec<ToolResult>>,
}

// ToolResult - result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
}

// MockFunction - function info in a tool call
#[derive(Debug, Clone)]
pub struct MockFunction {
    pub name: String,
    pub arguments: String,  // JSON string
}

// MockToolCall - a tool call from the LLM
#[derive(Debug, Clone)]
pub struct MockToolCall {
    pub function: MockFunction,
    pub id: String,
}

// MockResponse - the LLM's response
#[derive(Debug, Clone)]
pub struct MockResponse {
    pub thinking: String,
    pub content: String,
    pub tool_calls: Vec<MockToolCall>,
    pub raw: String,
    pub stop_reason: String,
}
