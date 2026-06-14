use std::sync::Arc;

use crate::agent::outcome::ToolCall;
use crate::llm::client::{LlmClient, ToolSchema};
use crate::llm::models::{ContentBlock, Message, Role, ToolResult};
use crate::tools::{ToolContext, ToolHandler};

pub struct AgentLoop {
    pub max_turns: u32,
    pub verbose: bool,
}

impl AgentLoop {
    pub fn new(max_turns: u32) -> Self {
        Self {
            max_turns,
            verbose: true,
        }
    }

    pub async fn run(
        &self,
        client: Arc<dyn LlmClient>,
        system_prompt: String,
        user_input: String,
        tools: Vec<Arc<dyn ToolHandler>>,
        tools_schema: Vec<ToolSchema>,
        mut context: ToolContext,
    ) -> Result<RunResult, AgentError> {
        let mut messages = vec![
            Message {
                role: Role::System,
                content: vec![ContentBlock::Text {
                    text: system_prompt,
                }],
                tool_results: None,
            },
            Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: user_input }],
                tool_results: None,
            },
        ];

        let mut turn = 0u32;

        while turn < self.max_turns {
            turn += 1;
            context.current_turn = turn;

            // Call LLM
            let response = client
                .chat(messages.clone(), Some(tools_schema.clone()))
                .await
                .map_err(|e| AgentError::LlmError(e.to_string()))?;

            // Parse tool calls
            let tool_calls = if response.tool_calls.is_empty() {
                // No tool calls - model responded with text
                // Return the content as the result
                if !response.content.is_empty() {
                    return Ok(RunResult {
                        reason: ExitReason::CurrentTaskDone {
                            data: Some(serde_json::Value::String(response.content.clone())),
                        },
                        turns: turn,
                    });
                }
                vec![ToolCall {
                    tool_name: "no_tool".to_string(),
                    args: serde_json::json!({}),
                    id: None,
                }]
            } else {
                response
                    .tool_calls
                    .iter()
                    .map(|tc| ToolCall {
                        tool_name: tc.function.name.clone(),
                        args: serde_json::from_str(&tc.function.arguments)
                            .unwrap_or(serde_json::json!({})),
                        id: Some(tc.id.clone()),
                    })
                    .collect()
            };

            // Execute tools
            let mut tool_results = Vec::new();
            let mut next_prompts = Vec::new();
            let mut exit_reason = None;

            for tc in tool_calls {
                // Handle no_tool case - model didn't call any tool, meaning task is done
                if tc.tool_name == "no_tool" {
                    exit_reason = Some(ExitReason::CurrentTaskDone { data: None });
                    break;
                }

                // Normalize tool name for common aliases
                let tool_name = normalize_tool_name(&tc.tool_name);

                let handler = tools
                    .iter()
                    .find(|t| t.name() == tool_name)
                    .ok_or_else(|| AgentError::ToolError(format!("unknown tool: {}", tc.tool_name)))?;

                let outcome = handler
                    .execute(tc.args, &mut context)
                    .await
                    .map_err(|e| AgentError::ToolError(e.to_string()))?;

                // Always capture tool result before checking exit conditions
                if let Some(id) = &tc.id {
                    let content = match &outcome.data {
                        Some(serde_json::Value::String(s)) => s.clone(),
                        Some(other) => other.to_string(),
                        None => "null".to_string(),
                    };
                    tool_results.push(ToolResult {
                        tool_use_id: id.clone(),
                        content,
                    });
                }

                if outcome.should_exit {
                    exit_reason = Some(ExitReason::Exited {
                        data: outcome.data,
                    });
                    break;
                }

                if outcome.next_prompt.is_none() {
                    exit_reason = Some(ExitReason::CurrentTaskDone {
                        data: outcome.data,
                    });
                    break;
                }

                if let Some(prompt) = outcome.next_prompt {
                    next_prompts.push(prompt);
                }
            }

            if let Some(reason) = exit_reason {
                return Ok(RunResult {
                    reason,
                    turns: turn,
                });
            }

            // Build next message
            let next_prompt = next_prompts.join("\n");
            messages.push(Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: next_prompt }],
                tool_results: Some(tool_results),
            });
        }

        Ok(RunResult {
            reason: ExitReason::MaxTurnsExceeded,
            turns: turn,
        })
    }
}

#[derive(Debug)]
pub struct RunResult {
    pub reason: ExitReason,
    pub turns: u32,
}

#[derive(Debug, Clone)]
pub enum ExitReason {
    Exited { data: Option<serde_json::Value> },
    CurrentTaskDone { data: Option<serde_json::Value> },
    MaxTurnsExceeded,
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("LLM error: {0}")]
    LlmError(String),
    #[error("Tool error: {0}")]
    ToolError(String),
}

/// Normalize tool name to handle common aliases
fn normalize_tool_name(name: &str) -> &str {
    match name {
        "write_to_file" | "write_file" | "create_file" => "file_write",
        "read_file" | "get_file" | "file_read_tool" | "file_reader" | "read" => "file_read",
        "patch" | "edit_file" | "modify_file" | "patch_file" | "replace" => "file_patch",
        "run" | "execute" | "shell" => "code_run",
        "ask" | "question" => "ask_user",
        _ => name,
    }
}
