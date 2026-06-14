use async_trait::async_trait;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::agent::outcome::StepOutcome;
use super::{ToolContext, ToolError, ToolHandler};

pub struct FileReadTool;
pub struct FilePatchTool;
pub struct FileWriteTool;

#[async_trait]
impl ToolHandler for FileReadTool {
    fn name(&self) -> &'static str { "file_read" }

    async fn execute(
        &self,
        args: Value,
        context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError> {
        let path_str = args.get("path")
            .or_else(|| args.get("file_path"))
            .or_else(|| args.get("filepath"))
            .or_else(|| args.get("filename"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("path required".to_string()))?;

        let path = resolve_path(path_str, &context.working_dir);
        let content = fs::read_to_string(&path)?;

        let start = args.get("start").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(200) as usize;

        let lines: Vec<&str> = content.lines().collect();
        let end = (start + count - 1).min(lines.len());
        let selected = &lines[start.saturating_sub(1)..end];

        let result = selected.iter().enumerate()
            .map(|(i, line)| format!("{:4} | {}", start + i, line))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(StepOutcome::done(Some(Value::String(result))))
    }
}

#[async_trait]
impl ToolHandler for FilePatchTool {
    fn name(&self) -> &'static str { "file_patch" }

    async fn execute(
        &self,
        args: Value,
        context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError> {
        let path_str = args.get("path")
            .or_else(|| args.get("file_path"))
            .or_else(|| args.get("filepath"))
            .or_else(|| args.get("filename"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("path required".to_string()))?;
        let old_content = args.get("old_content")
            .or_else(|| args.get("old"))
            .or_else(|| args.get("old_string"))
            .or_else(|| args.get("search"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("old_content required".to_string()))?;
        let new_content = args.get("new_content")
            .or_else(|| args.get("new"))
            .or_else(|| args.get("new_string"))
            .or_else(|| args.get("replace"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("new_content required".to_string()))?;

        let path = resolve_path(path_str, &context.working_dir);
        let content = fs::read_to_string(&path)?;

        let matches: Vec<_> = content.match_indices(old_content).collect();
        if matches.len() != 1 {
            return Err(ToolError::ExecutionFailed(
                format!("Expected 1 match, found {}", matches.len())
            ));
        }

        let new_text = content.replacen(old_content, new_content, 1);
        fs::write(&path, new_text)?;

        Ok(StepOutcome::done(Some(Value::String("patched".to_string()))))
    }
}

#[async_trait]
impl ToolHandler for FileWriteTool {
    fn name(&self) -> &'static str { "file_write" }

    async fn execute(
        &self,
        args: Value,
        context: &mut ToolContext,
    ) -> Result<StepOutcome, ToolError> {
        // Support multiple parameter name variants
        let path_str = args.get("path")
            .or_else(|| args.get("file_path"))
            .or_else(|| args.get("filepath"))
            .or_else(|| args.get("filename"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("path required".to_string()))?;
        let content = args.get("content")
            .or_else(|| args.get("contents"))
            .or_else(|| args.get("text"))
            .or_else(|| args.get("data"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("content required".to_string()))?;
        let mode = args.get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("overwrite");

        let path = resolve_path(path_str, &context.working_dir);

        match mode {
            "overwrite" => fs::write(&path, content)?,
            "append" => {
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(&path)?;
                file.write_all(content.as_bytes())?;
            },
            "prepend" => {
                let existing = fs::read_to_string(&path).unwrap_or_default();
                fs::write(&path, format!("{}{}", content, existing))?;
            },
            _ => return Err(ToolError::InvalidArgs(format!("invalid mode: {}", mode))),
        }

        Ok(StepOutcome::done(Some(Value::String("written".to_string()))))
    }
}

fn resolve_path(path_str: &str, working_dir: &Path) -> std::path::PathBuf {
    let path = Path::new(path_str);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        working_dir.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn test_context(dir: &TempDir) -> ToolContext {
        ToolContext {
            current_turn: 1,
            working_dir: dir.path().to_path_buf(),
            working_memory: super::super::WorkingMemory {
                key_info: None,
                related_sop: None,
                in_plan_mode: None,
                passed_sessions: 0,
            },
            verbose: false,
            project_root: dir.path().to_path_buf(),
        }
    }

    #[tokio::test]
    async fn test_file_read() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "line1\nline2\nline3\n").unwrap();

        let tool = FileReadTool;
        let mut ctx = test_context(&dir);
        let args = serde_json::json!({"path": "test.txt", "start": 1, "count": 2});
        let result = tool.execute(args, &mut ctx).await.unwrap();

        let data = result.data.unwrap().as_str().unwrap().to_string();
        assert!(data.contains("line1"));
        assert!(data.contains("line2"));
        assert!(!data.contains("line3"));
    }

    #[tokio::test]
    async fn test_file_patch() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "hello world").unwrap();

        let tool = FilePatchTool;
        let mut ctx = test_context(&dir);
        let args = serde_json::json!({
            "path": "test.txt",
            "old_content": "world",
            "new_content": "rust"
        });
        tool.execute(args, &mut ctx).await.unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello rust");
    }

    #[tokio::test]
    async fn test_file_write_overwrite() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");

        let tool = FileWriteTool;
        let mut ctx = test_context(&dir);
        let args = serde_json::json!({
            "path": "test.txt",
            "content": "hello rust",
            "mode": "overwrite"
        });
        tool.execute(args, &mut ctx).await.unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello rust");
    }
}
