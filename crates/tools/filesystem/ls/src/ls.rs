use async_trait::async_trait;
use loopal_error::LoopalError;
use loopal_tool_api::{PermissionLevel, Tool, ToolContext, ToolResult};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::ls_format;

pub struct LsTool;

#[async_trait]
impl Tool for LsTool {
    fn name(&self) -> &str {
        "Ls"
    }

    fn description(&self) -> &str {
        "List directory contents. Use long mode for size, permissions, and mtime. \
         When path points to a file, shows detailed file info (stat)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory or file path (default: cwd)"
                },
                "long": {
                    "type": "boolean",
                    "description": "Show permissions, size, and modification time (default: false)"
                },
                "all": {
                    "type": "boolean",
                    "description": "Include hidden files starting with '.' (default: false)"
                }
            }
        })
    }

    fn permission(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<ToolResult, LoopalError> {
        let target = resolve_path(&input, ctx);
        let md = tokio::fs::metadata(&target).await.map_err(|e| {
            LoopalError::Tool(loopal_error::ToolError::ExecutionFailed(format!(
                "Failed to access {}: {e}",
                target.display()
            )))
        })?;

        // Single file → stat-like output
        if !md.is_dir() {
            return Ok(ToolResult::success(ls_format::format_stat(&target, &md)));
        }

        let long = input["long"].as_bool().unwrap_or(false);
        let show_all = input["all"].as_bool().unwrap_or(false);
        list_directory(&target, long, show_all).await
    }
}

fn resolve_path(input: &Value, ctx: &ToolContext) -> PathBuf {
    match input["path"].as_str() {
        Some(p) => {
            let pb = PathBuf::from(p);
            if pb.is_absolute() { pb } else { ctx.cwd.join(pb) }
        }
        None => ctx.cwd.clone(),
    }
}

fn type_indicator(ft: Option<&std::fs::FileType>) -> &'static str {
    match ft {
        Some(ft) if ft.is_dir() => "/",
        Some(ft) if ft.is_symlink() => "@",
        _ => "",
    }
}

async fn list_directory(
    dir: &std::path::Path,
    long: bool,
    show_all: bool,
) -> Result<ToolResult, LoopalError> {
    let mut entries: Vec<(String, String)> = Vec::new();
    let mut read_dir = tokio::fs::read_dir(dir).await.map_err(|e| {
        LoopalError::Tool(loopal_error::ToolError::ExecutionFailed(format!(
            "Failed to read directory {}: {e}",
            dir.display()
        )))
    })?;

    while let Some(entry) = read_dir.next_entry().await.map_err(|e| {
        LoopalError::Tool(loopal_error::ToolError::ExecutionFailed(format!(
            "Failed to read entry: {e}",
        )))
    })? {
        let name = entry.file_name().to_string_lossy().to_string();
        if !show_all && name.starts_with('.') {
            continue;
        }
        let ft = entry.file_type().await.ok();
        let indicator = type_indicator(ft.as_ref());

        let display = if long {
            let md = entry.metadata().await.ok();
            ls_format::format_long_entry(&name, indicator, md.as_ref())
        } else {
            format!("{name}{indicator}")
        };
        entries.push((name.to_lowercase(), display));
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    if entries.is_empty() {
        Ok(ToolResult::success("(empty directory)"))
    } else {
        let lines: Vec<&str> = entries.iter().map(|(_, d)| d.as_str()).collect();
        Ok(ToolResult::success(lines.join("\n")))
    }
}
