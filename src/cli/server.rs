// cli/server.rs - Server command implementation for HTTP API

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{extract::State, routing::get, routing::post, Json, Router};
use serde::Serialize;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::api::routes::{AppState as BaseAppState, RunRequest, RunResponse};
use crate::config::manager::ConfigManager;
use crate::core::executor::LocalExecutor;
use crate::llm::client::ToolSchema;
use crate::tools::ToolHandler;

/// Server command arguments
#[derive(Debug, clap::Args)]
pub struct ServeArgs {
    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

/// Extended app state with config manager
#[derive(Clone)]
pub struct ServerAppState {
    pub base: BaseAppState,
    pub config_manager: ConfigManager,
}

/// Config response
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub default_provider: String,
    pub providers: Vec<String>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Handle the serve command
pub async fn handle_serve_command(args: ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Create config manager and load configuration
    let config_manager = ConfigManager::new();

    if let Some(config_path) = &args.config {
        config_manager
            .load_from_file(config_path)
            .await
            .map_err(|e| format!("Failed to load config: {}", e))?;
        println!("Loaded configuration from: {}", config_path.display());
    } else if let Some(default_config) = crate::cli::default_config_path() {
        if default_config.exists() {
            let _ = config_manager.load_from_file(&default_config).await;
            println!("Loaded configuration from: {}", default_config.display());
        }
    }

    // Build tools
    let tools: Vec<Arc<dyn ToolHandler>> = vec![
        Arc::new(crate::tools::file_ops::FileReadTool),
        Arc::new(crate::tools::file_ops::FilePatchTool),
        Arc::new(crate::tools::file_ops::FileWriteTool),
        Arc::new(crate::tools::code_run::CodeRunTool),
        Arc::new(crate::tools::memory::UpdateWorkingCheckpointTool),
        Arc::new(crate::tools::memory::StartLongTermUpdateTool),
        Arc::new(crate::tools::user::AskUserTool),
    ];

    let tools_schema = load_tools_schema();

    let base_state = BaseAppState {
        tools,
        tools_schema,
    };

    let state = ServerAppState {
        base: base_state,
        config_manager,
    };

    // Create router with all routes
    let app = create_server_router(state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    println!("CX server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create server router with all routes
fn create_server_router(state: ServerAppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Agent run endpoint
        .route("/api/run", post(api_run))
        // Config endpoints
        .route("/api/config", get(get_config))
        .route("/api/config/reload", post(reload_config))
        // Schema endpoint
        .route("/api/schema", get(get_schema))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

/// Health check handler
async fn health_check() -> &'static str {
    "OK"
}

/// Get schema handler
async fn get_schema(State(state): State<ServerAppState>) -> Json<Vec<ToolSchema>> {
    Json(state.base.tools_schema.clone())
}

/// Run agent handler
async fn api_run(
    State(state): State<ServerAppState>,
    Json(req): Json<RunRequest>,
) -> Result<Json<RunResponse>, Json<ErrorResponse>> {
    // Create executor with config manager
    let executor = LocalExecutor::with_config_manager(state.config_manager.clone())
        .with_max_turns(req.max_turns)
        .with_verbose(req.verbose);

    // Get working directory
    let working_dir = std::env::current_dir().map_err(|e| {
        Json(ErrorResponse {
            error: format!("Failed to get working directory: {}", e),
        })
    })?;

    // Execute task
    let result = executor
        .execute(
            &req.user_input,
            Some(&req.system_prompt),
            None, // Use default provider
            state.base.tools.clone(),
            state.base.tools_schema.clone(),
            working_dir,
        )
        .await
        .map_err(|e| Json(ErrorResponse { error: e.to_string() }))?;

    // Format response
    let (result_str, data) = match result.reason {
        crate::agent::r#loop::ExitReason::Exited { data } => ("EXITED".to_string(), data),
        crate::agent::r#loop::ExitReason::CurrentTaskDone { data } => {
            ("CURRENT_TASK_DONE".to_string(), data)
        }
        crate::agent::r#loop::ExitReason::MaxTurnsExceeded => {
            ("MAX_TURNS_EXCEEDED".to_string(), None)
        }
    };

    Ok(Json(RunResponse {
        result: result_str,
        data,
        turns: result.turns,
    }))
}

/// Get config handler
async fn get_config(State(state): State<ServerAppState>) -> Result<Json<ConfigResponse>, Json<ErrorResponse>> {
    let config = state.config_manager.get_config().await;
    let providers = state.config_manager.get_provider_names().await;

    Ok(Json(ConfigResponse {
        default_provider: config.global.default_provider,
        providers,
    }))
}

/// Reload config handler
async fn reload_config(State(state): State<ServerAppState>) -> Result<Json<serde_json::Value>, Json<ErrorResponse>> {
    match state.config_manager.reload().await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "success",
            "message": "Configuration reloaded"
        }))),
        Err(e) => Err(Json(ErrorResponse {
            error: format!("Failed to reload config: {}", e),
        })),
    }
}

/// Load tools schema from assets
fn load_tools_schema() -> Vec<ToolSchema> {
    let schema_path = crate::utils::assets_dir().join("tools_schema.json");

    if !schema_path.exists() {
        return vec![];
    }

    let content = std::fs::read_to_string(&schema_path).unwrap_or_default();
    if content.is_empty() {
        return vec![];
    }

    serde_json::from_str(&content).unwrap_or_default()
}
