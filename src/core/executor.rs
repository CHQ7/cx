// core/executor.rs - Local executor for running agent tasks

use std::path::PathBuf;
use std::sync::Arc;

use crate::agent::r#loop::{AgentLoop, RunResult};
use crate::config::manager::ConfigManager;
use crate::config::model::ProviderConfig;
use crate::llm::claude::ClaudeSession;
use crate::llm::client::{LlmClient, ToolSchema};
use crate::llm::openai::OpenAiSession;
use crate::tools::{ToolContext, ToolHandler, WorkingMemory};

/// Execution error types
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("LLM client error: {0}")]
    Llm(String),
    #[error("Tool execution error: {0}")]
    Tool(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Local executor that runs agent tasks
pub struct LocalExecutor {
    config_manager: ConfigManager,
    max_turns: u32,
    verbose: bool,
}

impl LocalExecutor {
    /// Create a new local executor with default configuration
    pub fn new() -> Self {
        Self {
            config_manager: ConfigManager::new(),
            max_turns: 40,
            verbose: true,
        }
    }

    /// Create a new local executor with a config manager
    pub fn with_config_manager(config_manager: ConfigManager) -> Self {
        Self {
            config_manager,
            max_turns: 40,
            verbose: true,
        }
    }

    /// Set max turns
    pub fn with_max_turns(mut self, max_turns: u32) -> Self {
        self.max_turns = max_turns;
        self
    }

    /// Set verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Load configuration from file
    pub async fn load_config<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), ExecutorError> {
        self.config_manager
            .load_from_file(path)
            .await
            .map_err(|e| ExecutorError::Config(e.to_string()))
    }

    /// Create LLM client from provider configuration
    fn create_llm_client(&self, provider: &ProviderConfig) -> Result<Arc<dyn LlmClient>, ExecutorError> {
        match provider {
            ProviderConfig::Claude(config) => {
                let api_key = config
                    .api_key
                    .clone()
                    .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .ok_or_else(|| ExecutorError::Config("Anthropic API key not found".to_string()))?;

                Ok(Arc::new(ClaudeSession::new(
                    api_key,
                    config.base_url.clone(),
                    config.default_model.clone(),
                )))
            }
            ProviderConfig::OpenAi(config) => {
                // For local models (Ollama, LM Studio, etc), API key is optional
                let is_local = config.base_url.contains("localhost")
                    || config.base_url.contains("127.0.0.1")
                    || config.base_url.contains(":11434")  // Ollama
                    || config.base_url.contains(":1234"); // LM Studio

                let api_key = if is_local {
                    // Local models: use config key, env key, or empty string
                    config
                        .api_key
                        .clone()
                        .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                        .unwrap_or_default()
                } else {
                    // Cloud services: require API key
                    config
                        .api_key
                        .clone()
                        .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                        .ok_or_else(|| ExecutorError::Config("OpenAI API key not found".to_string()))?
                };

                Ok(Arc::new(OpenAiSession::new(
                    api_key,
                    config.base_url.clone(),
                    config.default_model.clone(),
                )))
            }
            ProviderConfig::Mixin(_) => {
                // For mixin providers, we'd need to implement a composite client
                // For now, return an error
                Err(ExecutorError::Config(
                    "Mixin providers not yet supported in LocalExecutor".to_string(),
                ))
            }
        }
    }

    /// Create LLM client by provider name
    pub async fn create_client_for_provider(
        &self,
        provider_name: Option<&str>,
    ) -> Result<Arc<dyn LlmClient>, ExecutorError> {
        let provider = if let Some(name) = provider_name {
            self.config_manager
                .get_provider(name)
                .await
                .ok_or_else(|| ExecutorError::Config(format!("Provider '{}' not found", name)))?
        } else {
            self.config_manager
                .get_default_provider()
                .await
                .map(|(_, p)| p)
                .ok_or_else(|| ExecutorError::Config("No default provider configured".to_string()))?
        };

        self.create_llm_client(&provider)
    }

    /// Build default tool chain
    pub fn build_default_tools() -> Vec<Arc<dyn ToolHandler>> {
        use crate::tools::code_run::CodeRunTool;
        use crate::tools::file_ops::{FilePatchTool, FileReadTool, FileWriteTool};
        use crate::tools::memory::{StartLongTermUpdateTool, UpdateWorkingCheckpointTool};
        use crate::tools::user::AskUserTool;

        vec![
            Arc::new(FileReadTool),
            Arc::new(FilePatchTool),
            Arc::new(FileWriteTool),
            Arc::new(CodeRunTool),
            Arc::new(UpdateWorkingCheckpointTool),
            Arc::new(StartLongTermUpdateTool),
            Arc::new(AskUserTool),
        ]
    }

    /// Build tool chain with all tools including web tools
    pub fn build_full_tools() -> Vec<Arc<dyn ToolHandler>> {
        use crate::tools::code_run::CodeRunTool;
        use crate::tools::file_ops::{FilePatchTool, FileReadTool, FileWriteTool};
        use crate::tools::memory::{StartLongTermUpdateTool, UpdateWorkingCheckpointTool};
        use crate::tools::user::AskUserTool;
        use crate::tools::web::{WebExecuteJsTool, WebScanTool};

        vec![
            Arc::new(FileReadTool),
            Arc::new(FilePatchTool),
            Arc::new(FileWriteTool),
            Arc::new(CodeRunTool),
            Arc::new(WebScanTool { driver: None }),
            Arc::new(WebExecuteJsTool { driver: None }),
            Arc::new(UpdateWorkingCheckpointTool),
            Arc::new(StartLongTermUpdateTool),
            Arc::new(AskUserTool),
        ]
    }

    /// Execute a task with the given prompt
    pub async fn execute(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        provider_name: Option<&str>,
        tools: Vec<Arc<dyn ToolHandler>>,
        tools_schema: Vec<ToolSchema>,
        working_dir: PathBuf,
    ) -> Result<RunResult, ExecutorError> {
        // Create LLM client
        let client = self.create_client_for_provider(provider_name).await?;

        // Build tool context
        let context = ToolContext {
            current_turn: 0,
            working_dir: working_dir.clone(),
            working_memory: WorkingMemory {
                key_info: None,
                related_sop: None,
                in_plan_mode: None,
                passed_sessions: 0,
            },
            verbose: self.verbose,
            project_root: working_dir,
        };

        // Use default system prompt if none provided
        let system = system_prompt
            .map(|s| s.to_string())
            .unwrap_or_else(|| Self::default_system_prompt());

        // Create and run agent loop
        let agent_loop = AgentLoop::new(self.max_turns);

        agent_loop
            .run(client, system, prompt.to_string(), tools, tools_schema, context)
            .await
            .map_err(|e| ExecutorError::Llm(e.to_string()))
    }

    /// Execute with minimal setup (uses default tools and system prompt)
    pub async fn execute_simple(
        &self,
        prompt: &str,
        provider_name: Option<&str>,
        working_dir: PathBuf,
    ) -> Result<RunResult, ExecutorError> {
        let tools = Self::build_default_tools();
        let tools_schema = load_tools_schema();

        self.execute(
            prompt,
            None,
            provider_name,
            tools,
            tools_schema,
            working_dir,
        )
        .await
    }

    /// Default system prompt
    fn default_system_prompt() -> String {
        r#"You are CX, an AI assistant that helps users with various tasks.

You have access to tools for:
- Reading, writing, and patching files
- Running code (Python, shell scripts)
- Managing working memory
- Asking the user for clarification

When given a task:
1. Analyze what needs to be done
2. Use the appropriate tools to accomplish it
3. Report back when complete

Be concise and focused in your responses."#
            .to_string()
    }
}

impl Default for LocalExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Load tools schema - inline definition to avoid external file dependency
fn load_tools_schema() -> Vec<ToolSchema> {
    vec![
        ToolSchema {
            name: "file_write".to_string(),
            description: "Write content to a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
        },
        ToolSchema {
            name: "file_read".to_string(),
            description: "Read content from a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolSchema {
            name: "file_patch".to_string(),
            description: "Apply a patch to a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to patch"
                    },
                    "old_string": {
                        "type": "string",
                        "description": "String to search for"
                    },
                    "new_string": {
                        "type": "string",
                        "description": "Replacement string"
                    }
                },
                "required": ["path", "old_string", "new_string"]
            }),
        },
        ToolSchema {
            name: "code_run".to_string(),
            description: "Execute code or shell command".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "description": "Programming language or 'shell'"
                    },
                    "code": {
                        "type": "string",
                        "description": "Code to execute"
                    }
                },
                "required": ["language", "code"]
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_new() {
        let executor = LocalExecutor::new();
        assert_eq!(executor.max_turns, 40);
        assert!(executor.verbose);
    }

    #[test]
    fn test_executor_with_max_turns() {
        let executor = LocalExecutor::new().with_max_turns(20);
        assert_eq!(executor.max_turns, 20);
    }

    #[test]
    fn test_executor_with_verbose() {
        let executor = LocalExecutor::new().with_verbose(false);
        assert!(!executor.verbose);
    }

    #[test]
    fn test_build_default_tools() {
        let tools = LocalExecutor::build_default_tools();
        assert!(!tools.is_empty());

        // Check that we have the expected tools
        let tool_names: Vec<_> = tools.iter().map(|t| t.name()).collect();
        assert!(tool_names.contains(&"file_read"));
        assert!(tool_names.contains(&"file_write"));
        assert!(tool_names.contains(&"code_run"));
    }
}
