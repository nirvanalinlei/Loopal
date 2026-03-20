pub mod copy;
pub mod delete;
pub mod move_file;

use loopal_error::LoopalError;
use std::path::{Path, PathBuf};

/// Resolve a user-supplied path and reject traversal outside `cwd`.
pub fn resolve_and_guard(raw: &str, cwd: &Path) -> Result<PathBuf, LoopalError> {
    let pb = PathBuf::from(raw);
    let resolved = if pb.is_absolute() { pb } else { cwd.join(pb) };

    // Canonicalize the parent to check traversal (file itself may not exist yet)
    let parent = resolved.parent().unwrap_or(&resolved);
    if let Ok(canonical) = parent.canonicalize()
        && !canonical.starts_with(cwd)
        && !resolved.starts_with(cwd)
    {
        return Err(LoopalError::Tool(loopal_error::ToolError::InvalidInput(
            format!("path outside working directory: {}", resolved.display()),
        )));
    }
    Ok(resolved)
}

/// Extract a required string parameter from JSON input.
pub fn require_str<'a>(input: &'a serde_json::Value, key: &str) -> Result<&'a str, LoopalError> {
    input[key].as_str().ok_or_else(|| {
        LoopalError::Tool(loopal_error::ToolError::InvalidInput(format!(
            "{key} is required"
        )))
    })
}
