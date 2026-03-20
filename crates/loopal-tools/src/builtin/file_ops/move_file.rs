use async_trait::async_trait;
use loopal_error::LoopalError;
use loopal_tool_api::{PermissionLevel, Tool, ToolContext, ToolResult};
use serde_json::{json, Value};

use super::{require_str, resolve_and_guard};

pub struct MoveFileTool;

#[async_trait]
impl Tool for MoveFileTool {
    fn name(&self) -> &str {
        "MoveFile"
    }

    fn description(&self) -> &str {
        "Move or rename a file or directory."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["src", "dst"],
            "properties": {
                "src": { "type": "string", "description": "Source path" },
                "dst": { "type": "string", "description": "Destination path" }
            }
        })
    }

    fn permission(&self) -> PermissionLevel {
        PermissionLevel::Supervised
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<ToolResult, LoopalError> {
        let src_raw = require_str(&input, "src")?;
        let dst_raw = require_str(&input, "dst")?;
        let src = resolve_and_guard(src_raw, &ctx.cwd)?;
        let dst = resolve_and_guard(dst_raw, &ctx.cwd)?;

        if !src.exists() {
            return Ok(ToolResult::error(format!(
                "source does not exist: {}", src.display()
            )));
        }

        // If dst is an existing directory, move src inside it
        let final_dst = if dst.is_dir() {
            let name = src.file_name().ok_or_else(|| {
                LoopalError::Tool(loopal_error::ToolError::InvalidInput(
                    "source has no file name".into(),
                ))
            })?;
            dst.join(name)
        } else {
            dst
        };

        // Ensure parent directory exists
        if let Some(parent) = final_dst.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                LoopalError::Tool(loopal_error::ToolError::ExecutionFailed(format!(
                    "failed to create parent directory: {e}"
                )))
            })?;
        }

        // Try atomic rename first; fall back to copy+delete for cross-device moves
        if tokio::fs::rename(&src, &final_dst).await.is_err() {
            tokio::fs::copy(&src, &final_dst).await.map_err(|e| {
                LoopalError::Tool(loopal_error::ToolError::ExecutionFailed(format!(
                    "copy failed: {e}"
                )))
            })?;
            tokio::fs::remove_file(&src).await.map_err(|e| {
                LoopalError::Tool(loopal_error::ToolError::ExecutionFailed(format!(
                    "remove original failed: {e}"
                )))
            })?;
        }

        Ok(ToolResult::success(format!(
            "Moved {} → {}", src.display(), final_dst.display()
        )))
    }
}
