use async_trait::async_trait;
use std::path::PathBuf;
use thiserror::Error;

use crate::agent::outcome::StepOutcome;

pub mod code_run;
pub mod file_ops;
pub mod web;
pub mod memory;
pub mod user;

// ToolError - all possible tool errors
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// WorkingMemory - short-term working memory
#[derive(Debug, Clone)]
pub struct WorkingMemory {
    pub key_info: Option<String>,
    pub related_sop: Option<String>,
    pub in_plan_mode: Option<String>,
    pub passed_sessions: u32,
}

// ToolContext - context passed to each tool execution
#[derive(Debug)]
pub struct ToolContext {
    pub current_turn: u32,
    pub working_dir: PathBuf,
    pub working_memory: WorkingMemory,
    pub verbose: bool,
    pub project_root: PathBuf,
}

// ToolHandler trait - all tools implement this
#[async_trait]
pub trait ToolHandler: Send + Sync {
    fn name(&self) -> &'static str;

    async fn execute(
        &self,
        args: serde_json::Value,
        context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError>;
}
