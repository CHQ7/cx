use ga_core::api::{AppState, create_router};
use ga_core::llm::client::ToolSchema;
use ga_core::tools::file_ops::{FileReadTool, FilePatchTool, FileWriteTool};
use ga_core::tools::code_run::CodeRunTool;
use ga_core::tools::web::{WebScanTool, WebExecuteJsTool};
use ga_core::tools::memory::{UpdateWorkingCheckpointTool, StartLongTermUpdateTool};
use ga_core::tools::user::AskUserTool;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Load tools schema from JSON
    let tools_schema = load_tools_schema();

    let tools: Vec<Arc<dyn ga_core::tools::ToolHandler>> = vec![
        Arc::new(FileReadTool),
        Arc::new(FilePatchTool),
        Arc::new(FileWriteTool),
        Arc::new(CodeRunTool),
        Arc::new(WebScanTool { driver: None }),
        Arc::new(WebExecuteJsTool { driver: None }),
        Arc::new(UpdateWorkingCheckpointTool),
        Arc::new(StartLongTermUpdateTool),
        Arc::new(AskUserTool),
    ];

    let state = AppState { tools, tools_schema: tools_schema.clone() };
    let app = create_router(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("ga-core listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn load_tools_schema() -> Vec<ToolSchema> {
    let schema_path = ga_core::utils::assets_dir().join("tools_schema.json");

    if !schema_path.exists() {
        eprintln!("Warning: tools_schema.json not found at {:?}", schema_path);
        return vec![];
    }

    let content = std::fs::read_to_string(&schema_path)
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to read tools_schema.json: {}", e);
            String::new()
        });

    if content.is_empty() {
        return vec![];
    }

    serde_json::from_str(&content)
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to parse tools_schema.json: {}", e);
            vec![]
        })
}
