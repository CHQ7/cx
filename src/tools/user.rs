use async_trait::async_trait;
use serde_json::Value;

use crate::agent::outcome::StepOutcome;
use super::{ToolContext, ToolError, ToolHandler};

pub struct AskUserTool;

#[async_trait]
impl ToolHandler for AskUserTool {
    fn name(&self) -> &'static str { "ask_user" }

    async fn execute(
        &self,
        args: Value,
        _context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError> {
        let question = args.get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("Please provide input");
        let candidates = args.get("candidates")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        let result = serde_json::json!({
            "status": "INTERRUPT",
            "question": question,
            "candidates": candidates,
        });

        Ok(StepOutcome::done(Some(result)))
    }
}
