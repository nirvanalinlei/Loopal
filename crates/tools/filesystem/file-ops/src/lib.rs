pub mod copy;
pub mod delete;
pub mod move_file;

use loopal_error::LoopalError;

/// Extract a required string parameter from JSON input.
pub fn require_str<'a>(input: &'a serde_json::Value, key: &str) -> Result<&'a str, LoopalError> {
    input[key].as_str().ok_or_else(|| {
        LoopalError::Tool(loopal_error::ToolError::InvalidInput(format!(
            "{key} is required"
        )))
    })
}
