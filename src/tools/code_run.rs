use async_trait::async_trait;
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::agent::outcome::StepOutcome;
use super::{ToolContext, ToolError, ToolHandler};

pub struct CodeRunTool;

#[async_trait]
impl ToolHandler for CodeRunTool {
    fn name(&self) -> &'static str { "code_run" }

    async fn execute(
        &self,
        args: Value,
        _context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError> {
        let script = args.get("script")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let code_type = args.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("python");
        let timeout_secs = args.get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);
        let cwd = args.get("cwd")
            .and_then(|v| v.as_str())
            .map(|s| std::path::PathBuf::from(s));

        let result = match code_type {
            "python" | "py" => run_python(script, cwd.as_deref(), timeout_secs).await,
            "powershell" | "bash" | "sh" | "shell" | "ps1" | "pwsh" => {
                run_shell(script, code_type, cwd.as_deref(), timeout_secs).await
            },
            _ => return Err(ToolError::InvalidArgs(format!("unsupported type: {}", code_type))),
        };

        Ok(StepOutcome::done(Some(result?)))
    }
}

async fn run_python(
    script: &str,
    cwd: Option<&std::path::Path>,
    timeout_secs: u64,
) -> Result<Value, ToolError> {
    let mut cmd = Command::new("python");
    cmd.args(&["-c", script])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }

    let output = timeout(
        Duration::from_secs(timeout_secs),
        cmd.output(),
    ).await
    .map_err(|_| ToolError::ExecutionFailed("timeout".to_string()))?
    .map_err(|e| ToolError::Io(e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(serde_json::json!({
        "status": if output.status.success() { "success" } else { "error" },
        "stdout": stdout.to_string(),
        "stderr": stderr.to_string(),
        "exit_code": output.status.code(),
    }))
}

async fn run_shell(
    script: &str,
    shell_type: &str,
    cwd: Option<&std::path::Path>,
    timeout_secs: u64,
) -> Result<Value, ToolError> {
    let (shell_cmd, shell_args) = if cfg!(target_os = "windows") {
        match shell_type {
            "powershell" | "ps1" | "pwsh" => {
                let ps = if std::process::Command::new("pwsh").arg("--version").output().is_ok() {
                    "pwsh"
                } else {
                    "powershell"
                };
                (ps, vec!["-NoProfile", "-NonInteractive", "-Command", script])
            },
            _ => ("cmd", vec!["/C", script]),
        }
    } else {
        ("bash", vec!["-c", script])
    };

    let mut cmd = Command::new(shell_cmd);
    cmd.args(&shell_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }

    let output = timeout(
        Duration::from_secs(timeout_secs),
        cmd.output(),
    ).await
    .map_err(|_| ToolError::ExecutionFailed("timeout".to_string()))?
    .map_err(|e| ToolError::Io(e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(serde_json::json!({
        "status": if output.status.success() { "success" } else { "error" },
        "stdout": stdout.to_string(),
        "stderr": stderr.to_string(),
        "exit_code": output.status.code(),
    }))
}
