use clap::{Parser, Subcommand};

pub mod config_cmd;
pub mod run;
pub mod server;

/// CX - AI 助手命令行工具
#[derive(Parser, Debug)]
#[command(name = "cx")]
#[command(about = "CX - AI 助手命令行工具")]
#[command(version = "0.1.0")]
pub struct Cli {
    /// 配置文件路径
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,

    /// 启用调试模式
    #[arg(short, long)]
    pub debug: bool,

    /// 子命令
    #[command(subcommand)]
    pub command: Commands,
}

/// 可用子命令
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 配置管理命令
    #[command(subcommand)]
    Config(config_cmd::ConfigCommands),

    /// 启动交互式会话 (暂未实现)
    Chat {
        /// 初始提示词
        #[arg(short, long)]
        prompt: Option<String>,
    },

    /// 执行单次任务
    Run(run::RunArgs),

    /// 启动 HTTP API 服务
    Serve(server::ServeArgs),
}

/// 初始化 tracing 日志
pub fn init_tracing() {
    tracing_subscriber::fmt::init();
}

/// 获取默认配置文件路径
pub fn default_config_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|home| home.join(".cx").join("config.toml"))
}
