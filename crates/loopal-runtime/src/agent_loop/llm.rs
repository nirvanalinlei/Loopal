use std::time::Instant;

use futures::StreamExt;
use loopal_error::Result;
use loopal_message::Message;
use loopal_protocol::AgentEventPayload;
use loopal_provider_api::{ChatParams, StopReason, StreamChunk};
use tracing::{error, info, warn};

use super::runner::AgentLoopRunner;

impl AgentLoopRunner {
    /// Build chat params from a provided message slice (typically a working copy).
    pub fn prepare_chat_params_with(&self, messages: &[Message]) -> Result<ChatParams> {
        let full_system_prompt = format!(
            "{}{}",
            self.params.system_prompt,
            self.params.mode.system_prompt_suffix()
        );
        let mut tool_defs = self.params.kernel.tool_definitions();

        if let Some(ref filter) = self.params.tool_filter {
            tool_defs.retain(|t| filter.contains(&t.name));
        }

        Ok(ChatParams {
            model: self.params.model.clone(),
            messages: messages.to_vec(),
            system_prompt: full_system_prompt,
            tools: tool_defs,
            max_tokens: self.max_output_tokens,
            temperature: None,
            debug_dump_dir: Some(loopal_config::tmp_dir()),
        })
    }

    /// Stream the LLM response using a provided working copy of messages.
    ///
    /// Unlike the old `stream_llm`, this does NOT call preflight — the caller
    /// is responsible for running preflight on the working copy beforehand.
    /// Returns (assistant_text, tool_uses, stream_error, stop_reason).
    pub async fn stream_llm_with(
        &mut self,
        messages: &[Message],
    ) -> Result<(String, Vec<(String, String, serde_json::Value)>, bool, StopReason)> {
        let chat_params = self.prepare_chat_params_with(messages)?;
        let provider = self.params.kernel.resolve_provider(&self.params.model)?;

        let llm_start = Instant::now();
        info!(
            model = %self.params.model,
            messages = messages.len(),
            tools = chat_params.tools.len(),
            max_tokens = chat_params.max_tokens,
            "LLM request"
        );

        const MAX_RETRIES: u32 = 6;
        const BASE_WAIT_MS: u64 = 2000;
        let mut retry_count = 0;
        let mut stream = loop {
            match provider.stream_chat(&chat_params).await {
                Ok(s) => break s,
                Err(e) if e.is_retryable() && retry_count < MAX_RETRIES => {
                    retry_count += 1;
                    let base_wait = e.retry_after_ms().unwrap_or(BASE_WAIT_MS);
                    let wait_ms = base_wait * (1 << (retry_count - 1));
                    warn!(
                        retry = retry_count, max_retries = MAX_RETRIES, wait_ms,
                        error = %e, "retryable error, retrying after backoff"
                    );
                    self.emit(AgentEventPayload::Error {
                        message: format!(
                            "{}. Retrying in {:.1}s ({}/{})",
                            e, wait_ms as f64 / 1000.0, retry_count, MAX_RETRIES
                        ),
                    }).await?;
                    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        };

        let mut assistant_text = String::new();
        let mut tool_uses: Vec<(String, String, serde_json::Value)> = Vec::new();
        let mut stream_error = false;
        let mut stop_reason = StopReason::EndTurn;

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(StreamChunk::Text { text }) => {
                    assistant_text.push_str(&text);
                    self.emit(AgentEventPayload::Stream { text }).await?;
                }
                Ok(StreamChunk::ToolUse { id, name, input }) => {
                    self.emit(AgentEventPayload::ToolCall {
                        id: id.clone(), name: name.clone(), input: input.clone(),
                    }).await?;
                    tool_uses.push((id, name, input));
                }
                Ok(StreamChunk::Usage {
                    input_tokens, output_tokens,
                    cache_creation_input_tokens, cache_read_input_tokens,
                }) => {
                    self.total_input_tokens += input_tokens;
                    self.total_output_tokens += output_tokens;
                    self.total_cache_creation_tokens += cache_creation_input_tokens;
                    self.total_cache_read_tokens += cache_read_input_tokens;
                    self.emit(AgentEventPayload::TokenUsage {
                        input_tokens, output_tokens,
                        context_window: self.max_context_tokens,
                        cache_creation_input_tokens, cache_read_input_tokens,
                    }).await?;
                    info!(
                        total_input = self.total_input_tokens,
                        total_output = self.total_output_tokens,
                        context_window = self.max_context_tokens,
                        input_tokens, output_tokens,
                        cache_creation = cache_creation_input_tokens,
                        cache_read = cache_read_input_tokens,
                        "token usage"
                    );
                }
                Ok(StreamChunk::Done { stop_reason: reason }) => {
                    stop_reason = reason;
                    break;
                }
                Err(e) => {
                    error!(error = %e, turn = self.turn_count, model = %self.params.model, "stream error");
                    self.emit(AgentEventPayload::Error { message: e.to_string() }).await?;
                    stream_error = true;
                    break;
                }
            }
        }

        let llm_duration = llm_start.elapsed();
        info!(
            duration_ms = llm_duration.as_millis() as u64,
            tool_calls = tool_uses.len(),
            has_text = !assistant_text.is_empty(),
            "LLM complete"
        );

        Ok((assistant_text, tool_uses, stream_error, stop_reason))
    }
}
