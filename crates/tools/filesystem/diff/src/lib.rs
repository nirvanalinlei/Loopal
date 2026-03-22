use async_trait::async_trait;
use loopal_error::LoopalError;
use loopal_tool_api::{Backend, PermissionLevel, Tool, ToolContext, ToolResult};
use serde_json::{json, Value};

use loopal_edit_core::diff::{compute_diff, format_unified, DiffOp};

pub struct DiffTool;

const MAX_LINES: usize = 2000;

#[async_trait]
impl Tool for DiffTool {
    fn name(&self) -> &str {
        "Diff"
    }

    fn description(&self) -> &str {
        "Compare two files, or a file against a git ref. Returns unified diff output."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path_a": {
                    "type": "string",
                    "description": "First file path (for two-file mode)"
                },
                "path_b": {
                    "type": "string",
                    "description": "Second file path (for two-file mode)"
                },
                "path": {
                    "type": "string",
                    "description": "File path (for git mode)"
                },
                "ref": {
                    "type": "string",
                    "description": "Git ref to compare against (default: HEAD)"
                }
            }
        })
    }

    fn permission(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    async fn execute(&self, input: Value, ctx: &ToolContext) -> Result<ToolResult, LoopalError> {
        if let Some(path_a) = input["path_a"].as_str() {
            let path_b = input["path_b"].as_str().ok_or_else(|| {
                LoopalError::Tool(loopal_error::ToolError::InvalidInput(
                    "path_b is required when path_a is given".into(),
                ))
            })?;
            return diff_two_files(path_a, path_b, &*ctx.backend).await;
        }

        if let Some(path) = input["path"].as_str() {
            let git_ref = input["ref"].as_str().unwrap_or("HEAD");
            return diff_git(path, git_ref, &*ctx.backend).await;
        }

        Ok(ToolResult::error(
            "Provide path_a+path_b (two-file mode) or path+ref (git mode)",
        ))
    }
}

async fn diff_two_files(
    a_raw: &str,
    b_raw: &str,
    backend: &dyn Backend,
) -> Result<ToolResult, LoopalError> {
    let a_text = match backend.read_raw(a_raw).await {
        Ok(s) => s,
        Err(e) => return Ok(ToolResult::error(e.to_string())),
    };
    let b_text = match backend.read_raw(b_raw).await {
        Ok(s) => s,
        Err(e) => return Ok(ToolResult::error(e.to_string())),
    };
    diff_texts(&a_text, &b_text, a_raw, b_raw)
}

async fn diff_git(
    path_raw: &str,
    git_ref: &str,
    backend: &dyn Backend,
) -> Result<ToolResult, LoopalError> {
    let file_path = match backend.resolve_path(path_raw, false) {
        Ok(p) => p,
        Err(e) => return Ok(ToolResult::error(e.to_string())),
    };
    let cwd = backend.cwd();
    let rel = file_path
        .strip_prefix(cwd)
        .unwrap_or(&file_path)
        .to_string_lossy();
    let ref_spec = format!("{git_ref}:{rel}");
    let cmd = format!("git show {ref_spec}");

    let result = match backend.exec(&cmd, 30_000).await {
        Ok(r) => r,
        Err(e) => return Ok(ToolResult::error(format!("git show failed: {e}"))),
    };

    if result.exit_code != 0 {
        return Ok(ToolResult::error(format!(
            "git show failed: {}",
            result.stderr
        )));
    }

    let old_text = result.stdout;
    let new_text = match backend.read_raw(path_raw).await {
        Ok(s) => s,
        Err(e) => return Ok(ToolResult::error(e.to_string())),
    };
    diff_texts(&old_text, &new_text, &ref_spec, &file_path.display().to_string())
}

fn diff_texts(
    old: &str,
    new: &str,
    old_name: &str,
    new_name: &str,
) -> Result<ToolResult, LoopalError> {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    if old_lines.len() > MAX_LINES || new_lines.len() > MAX_LINES {
        return Ok(ToolResult::error(format!(
            "Files exceed {MAX_LINES} lines. Use Bash with `git diff` or `diff` for large files."
        )));
    }
    let ops = compute_diff(&old_lines, &new_lines);
    if ops.iter().all(|op| matches!(op, DiffOp::Equal(_))) {
        return Ok(ToolResult::success("No differences"));
    }
    Ok(ToolResult::success(format_unified(
        old_name, new_name, &ops, 3,
    )))
}
