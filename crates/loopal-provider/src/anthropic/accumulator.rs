use loopal_provider_api::StopReason;

/// Accumulates partial tool-use JSON fragments across streamed deltas.
#[derive(Default)]
pub(crate) struct ToolUseAccumulator {
    pub(crate) current_tool_id: Option<String>,
    pub(crate) current_tool_name: Option<String>,
    pub(crate) json_fragments: String,
    pub(crate) stop_reason: Option<StopReason>,
}

/// Accumulates thinking content across streamed deltas.
#[derive(Default)]
pub(crate) struct ThinkingAccumulator {
    pub(crate) active: bool,
    pub(crate) signature_fragments: String,
}
