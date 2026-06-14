use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::llm::client::ToolSchema;
use crate::tools::ToolHandler;

#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub system_prompt: String,
    pub user_input: String,
    #[serde(default = "default_max_turns")]
    pub max_turns: u32,
    #[serde(default = "default_verbose")]
    pub verbose: bool,
}

fn default_max_turns() -> u32 {
    40
}

fn default_verbose() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub result: String,
    pub data: Option<serde_json::Value>,
    pub turns: u32,
}

#[derive(Clone)]
pub struct AppState {
    pub tools: Vec<Arc<dyn ToolHandler>>,
    pub tools_schema: Vec<ToolSchema>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/agent/run", post(run_agent))
        .route("/api/schema", get(get_schema))
        .with_state(state)
}

async fn run_agent(
    State(_state): State<AppState>,
    Json(_req): Json<RunRequest>,
) -> Json<RunResponse> {
    // TODO: Wire up actual AgentLoop execution
    // For now, return a mock response
    Json(RunResponse {
        result: "CURRENT_TASK_DONE".to_string(),
        data: None,
        turns: 1,
    })
}

async fn get_schema(State(state): State<AppState>) -> Json<Vec<ToolSchema>> {
    Json(state.tools_schema.clone())
}
