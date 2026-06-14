// cli/run.rs - Run command implementation for executing single tasks

use std::io::Read;
use std::path::PathBuf;

use crate::config::manager::ConfigManager;
use crate::core::executor::LocalExecutor;

/// Run command arguments
#[derive(Debug, clap::Args)]
pub struct RunArgs {
    /// The prompt to execute (can be multiple words)
    #[arg(value_name = "PROMPT")]
    pub prompt: Vec<String>,

    /// Provider to use (defaults to configured default)
    #[arg(short, long)]
    pub model: Option<String>,

    /// Maximum number of turns
    #[arg(long, default_value = "40")]
    pub max_turns: u32,

    /// Read prompt from stdin
    #[arg(long)]
    pub stdin: bool,

    /// Working directory for the task
    #[arg(short, long)]
    pub working_dir: Option<PathBuf>,

    /// System prompt to use
    #[arg(long)]
    pub system_prompt: Option<String>,

    /// Disable verbose output
    #[arg(short, long)]
    pub quiet: bool,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

/// Handle the run command
pub async fn handle_run_command(args: RunArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Build the prompt
    let prompt = if args.stdin {
        // Read from stdin
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer.trim().to_string()
    } else if !args.prompt.is_empty() {
        // Join prompt arguments
        args.prompt.join(" ")
    } else {
        eprintln!("Error: No prompt provided. Use --stdin to read from stdin or provide prompt arguments.");
        std::process::exit(1);
    };

    if prompt.is_empty() {
        eprintln!("Error: Empty prompt provided.");
        std::process::exit(1);
    }

    // Determine working directory
    let working_dir = args.working_dir.unwrap_or_else(|| {
        std::env::current_dir().expect("Failed to get current directory")
    });

    // Load configuration
    let config_manager = ConfigManager::new();

    // Load config from file if specified
    if let Some(config_path) = &args.config {
        config_manager
            .load_from_file(config_path)
            .await
            .map_err(|e| format!("Failed to load config: {}", e))?;
    } else if let Some(default_config) = crate::cli::default_config_path() {
        // Try to load default config if it exists
        if default_config.exists() {
            let _ = config_manager.load_from_file(&default_config).await;
        }
    }

    // Create executor
    let executor = LocalExecutor::with_config_manager(config_manager)
        .with_max_turns(args.max_turns)
        .with_verbose(!args.quiet);

    if !args.quiet {
        println!("Executing task...");
        println!("Working directory: {}", working_dir.display());
        if let Some(model) = &args.model {
            println!("Using provider: {}", model);
        }
    }

    // Execute the task
    let result = executor
        .execute_simple(&prompt, args.model.as_deref(), working_dir)
        .await;

    match result {
        Ok(run_result) => {
            if !args.quiet {
                println!("\nTask completed in {} turns", run_result.turns);
                println!("Exit reason: {:?}", run_result.reason);
            }

            // Print any result data
            match &run_result.reason {
                crate::agent::r#loop::ExitReason::Exited { data } |
                crate::agent::r#loop::ExitReason::CurrentTaskDone { data } => {
                    if let Some(data) = data {
                        println!("{}", serde_json::to_string_pretty(data)?);
                    }
                }
                crate::agent::r#loop::ExitReason::MaxTurnsExceeded => {
                    if !args.quiet {
                        println!("Warning: Max turns exceeded");
                    }
                }
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
