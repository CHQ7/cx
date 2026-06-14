use async_trait::async_trait;
use serde_json::Value;

use crate::agent::outcome::StepOutcome;
use super::{ToolContext, ToolError, ToolHandler};

pub struct UpdateWorkingCheckpointTool;
pub struct StartLongTermUpdateTool;

#[async_trait]
impl ToolHandler for UpdateWorkingCheckpointTool {
    fn name(&self) -> &'static str { "update_working_checkpoint" }

    async fn execute(
        &self,
        args: Value,
        context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError> {
        let key_info = args.get("key_info").and_then(|v| v.as_str());
        let related_sop = args.get("related_sop").and_then(|v| v.as_str());

        if let Some(info) = key_info {
            context.working_memory.key_info = Some(info.to_string());
        }
        if let Some(sop) = related_sop {
            context.working_memory.related_sop = Some(sop.to_string());
        }

        Ok(StepOutcome::done(Some(serde_json::json!({"status": "updated"}))))
    }
}

#[async_trait]
impl ToolHandler for StartLongTermUpdateTool {
    fn name(&self) -> &'static str { "start_long_term_update" }

    async fn execute(
        &self,
        _args: Value,
        _context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError> {
        // TODO: Trigger long-term memory distillation
        // For now, just acknowledge
        Ok(StepOutcome::done(Some(serde_json::json!({"status": "started"}))))
    }
}
