use async_trait::async_trait;
use loopal_error::LoopalError;
use loopal_tool_api::{PermissionLevel, Tool, ToolContext, ToolResult};
use serde_json::{json, Value};

use crate::{require_str, resolve_and_guard};

pub struct CopyFileTool;

#[async_trait]
impl Tool for CopyFileTool {
    fn name(&self) -> &str {
        "CopyFile"
    }

    fn description(&self) -> &str {
        "Copy a file to a new location."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["src", "dst"],
            "properties": {
                "src": { "type": "string", "description": "Source file path" },
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
        if !src.is_file() {
            return Ok(ToolResult::error(
                "source must be a file (use Bash for directory copies)",
            ));
        }

        // If dst is an existing directory, copy into it
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

        let bytes = tokio::fs::copy(&src, &final_dst).await.map_err(|e| {
            LoopalError::Tool(loopal_error::ToolError::ExecutionFailed(format!(
                "copy failed: {e}"
            )))
        })?;

        Ok(ToolResult::success(format!(
            "Copied {} → {} ({} bytes)", src.display(), final_dst.display(), bytes
        )))
    }
}
