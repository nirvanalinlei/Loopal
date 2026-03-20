use loopal_provider::get_model_info;
use loopal_error::{AgentOutput, Result};
use loopal_protocol::AgentEventPayload;
use loopal_provider_api::StopReason;
use loopal_tool_api::ToolContext;
use tracing::{Instrument, info, info_span, warn};

use super::{AgentLoopParams, MAX_AUTO_CONTINUATIONS, TurnOutput};

/// Encapsulates the agent loop state and behavior.
pub struct AgentLoopRunner {
    pub params: AgentLoopParams,
    pub tool_ctx: ToolContext,
    pub turn_count: u32,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub total_cache_creation_tokens: u32,
    pub total_cache_read_tokens: u32,
    pub max_context_tokens: u32,
    pub max_output_tokens: u32,
}

impl AgentLoopRunner {
    pub fn new(params: AgentLoopParams) -> Self {
        let tool_ctx = ToolContext {
            cwd: std::path::PathBuf::from(&params.session.cwd),
            session_id: params.session.id.clone(),
            shared: params.shared.clone(),
        };
        let model_info = get_model_info(&params.model);
        let max_context_tokens = model_info.as_ref().map_or(200_000, |m| m.context_window);
        let max_output_tokens = model_info.as_ref().map_or(16_384, |m| m.max_output_tokens);
        Self {
            params, tool_ctx, turn_count: 0,
            total_input_tokens: 0, total_output_tokens: 0,
            total_cache_creation_tokens: 0, total_cache_read_tokens: 0,
            max_context_tokens, max_output_tokens,
        }
    }

    /// Main loop — orchestrates input, middleware, LLM, and tool execution.
    /// Guarantees `Finished` event is emitted regardless of exit path.
    /// Returns structured AgentOutput with result text and termination reason.
    pub async fn run(&mut self) -> Result<AgentOutput> {
        let span = info_span!("agent", session_id = %self.params.session.id);
        self.run_instrumented().instrument(span).await
    }

    /// Actual run logic, executed inside the `agent` span.
    async fn run_instrumented(&mut self) -> Result<AgentOutput> {
        info!(model = %self.params.model, "agent loop started");
        self.emit(AgentEventPayload::Started).await?;

        let result = self.run_loop().await;

        // Surface the fatal error to TUI before finishing
        if let Err(ref e) = result {
            let _ = self.emit(AgentEventPayload::Error {
                message: e.to_string(),
            }).await;
        }

        // Always emit Finished, even on error — TUI relies on this to exit "Working" state.
        let _ = self.emit(AgentEventPayload::Finished).await;

        result
    }

    /// One complete turn: LLM → [tools → LLM]* → returns when no tool calls.
    ///
    /// Each LLM call uses a **working copy** of `params.messages`:
    /// middleware compaction and preflight truncation only affect the copy,
    /// so the persistent history (`params.messages` + storage) is never polluted.
    pub(super) async fn execute_turn(&mut self) -> Result<TurnOutput> {
        let mut last_text = String::new();
        let mut continuation_count: u32 = 0;
        loop {
            // Clone persistent history → middleware → preflight → send to LLM
            let mut working = self.params.messages.clone();
            if !self.execute_middleware_on(&mut working).await? {
                return Ok(TurnOutput { output: last_text });
            }
            self.preflight_check_on(&mut working);
            let (text, tool_uses, stream_error, stop_reason) =
                self.stream_llm_with(&working).await?;

            // max_tokens + tool calls → tool arguments may be truncated, discard tools
            if stop_reason == StopReason::MaxTokens && !tool_uses.is_empty() {
                warn!("max_tokens hit with tool calls — discarding truncated tools");
                self.record_assistant_message(&text, &[]);
                if !text.is_empty() { last_text.clone_from(&text); }
                if continuation_count < MAX_AUTO_CONTINUATIONS {
                    continuation_count += 1;
                    self.emit(AgentEventPayload::AutoContinuation {
                        continuation: continuation_count,
                        max_continuations: MAX_AUTO_CONTINUATIONS,
                    }).await?;
                    continue;
                }
                return Ok(TurnOutput { output: last_text });
            }

            // Record to persistent history (params.messages + storage)
            self.record_assistant_message(&text, &tool_uses);
            if !text.is_empty() { last_text.clone_from(&text); }

            if stream_error && tool_uses.is_empty() && text.is_empty() {
                return Ok(TurnOutput { output: last_text });
            }

            // max_tokens + pure text → auto-continue
            if tool_uses.is_empty() && stop_reason == StopReason::MaxTokens {
                if continuation_count < MAX_AUTO_CONTINUATIONS {
                    continuation_count += 1;
                    self.emit(AgentEventPayload::AutoContinuation {
                        continuation: continuation_count,
                        max_continuations: MAX_AUTO_CONTINUATIONS,
                    }).await?;
                    continue;
                }
                return Ok(TurnOutput { output: last_text });
            }

            if tool_uses.is_empty() {
                return Ok(TurnOutput { output: text });
            }

            // Execute tools → results appended to persistent params.messages
            let completion_result = self.execute_tools(tool_uses).await?;
            self.inject_pending_messages().await;
            continuation_count = 0;

            if let Some(result) = completion_result {
                return Ok(TurnOutput { output: result });
            }
        }
    }

    /// Send an event payload via the frontend.
    pub async fn emit(&self, payload: AgentEventPayload) -> Result<()> {
        self.params.frontend.emit(payload).await
    }
}
