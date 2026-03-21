use async_trait::async_trait;
use loopal_error::LoopalError;
use loopal_tool_api::{PermissionLevel, Tool, ToolContext, ToolResult};
use serde_json::{json, Value};

use crate::{require_str, resolve_and_guard};

pub struct DeleteTool;

#[async_trait]
impl Tool for DeleteTool {
    fn name(&self) -> &str {
        "Delete"
    }

    fn description(&self) -> &str {
        "Delete a file or directory."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": { "type": "string", "description": "Path to delete" }
            }
        })
    }

    fn permission(&self) -> PermissionLevel {
        PermissionLevel::Supervised
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<ToolResult, LoopalError> {
        let path_raw = require_str(&input, "path")?;
        let path = resolve_and_guard(path_raw, &ctx.cwd)?;

        if !path.exists() {
            return Ok(ToolResult::error(format!(
                "path does not exist: {}", path.display()
            )));
        }

        let md = tokio::fs::metadata(&path).await.map_err(|e| {
            LoopalError::Tool(loopal_error::ToolError::ExecutionFailed(format!(
                "failed to read metadata: {e}"
            )))
        })?;

        if md.is_dir() {
            let count = std::fs::read_dir(&path).map(|rd| rd.count()).unwrap_or(0);
            tokio::fs::remove_dir_all(&path).await.map_err(|e| {
                LoopalError::Tool(loopal_error::ToolError::ExecutionFailed(format!(
                    "failed to remove directory: {e}"
                )))
            })?;
            Ok(ToolResult::success(format!(
                "Deleted {} (directory, {} entries)", path.display(), count
            )))
        } else {
            tokio::fs::remove_file(&path).await.map_err(|e| {
                LoopalError::Tool(loopal_error::ToolError::ExecutionFailed(format!(
                    "failed to remove file: {e}"
                )))
            })?;
            Ok(ToolResult::success(format!(
                "Deleted {} (file)", path.display()
            )))
        }
    }
}
