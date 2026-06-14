use serde_json::Value;

// ToolCall - parsed from LLM response
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub tool_name: String,
    pub args: Value,
    pub id: Option<String>,
}

// StepOutcome - result of executing a tool
#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub data: Option<Value>,
    pub next_prompt: Option<String>,
    pub should_exit: bool,
}

impl StepOutcome {
    pub fn exit(data: Option<Value>) -> Self {
        Self { data, next_prompt: None, should_exit: true }
    }

    pub fn continue_with(prompt: String, data: Option<Value>) -> Self {
        Self { data, next_prompt: Some(prompt), should_exit: false }
    }

    pub fn done(data: Option<Value>) -> Self {
        Self { data, next_prompt: None, should_exit: false }
    }
}

// ToolResult - result to send back to LLM
#[derive(Debug, Clone)]
pub struct AgentToolResult {
    pub tool_use_id: String,
    pub content: String,
}
