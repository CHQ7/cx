use clap::Parser;
use ga_core::cli::{Cli, Commands, default_config_path, init_tracing};
use ga_core::cli::config_cmd;
use ga_core::cli::run;
use ga_core::cli::server;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let cli = Cli::parse();

    // 确定配置文件路径
    let config_path = cli
        .config
        .map(PathBuf::from)
        .or_else(default_config_path)
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    match cli.command {
        Commands::Config(config_cmd_args) => {
            config_cmd::handle_config_command(config_cmd_args, &config_path).await?;
        }

        Commands::Chat { prompt: _ } => {
            println!("交互式会话模式暂未实现");
        }

        Commands::Run(args) => {
            run::handle_run_command(args).await?;
        }

        Commands::Serve(args) => {
            server::handle_serve_command(args).await?;
        }
    }

    Ok(())
}
