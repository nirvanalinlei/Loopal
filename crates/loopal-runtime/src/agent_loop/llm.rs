use std::time::Instant;

use futures::StreamExt;
use loopal_error::Result;
use loopal_message::Message;
use loopal_protocol::AgentEventPayload;
use loopal_provider::{get_thinking_capability, resolve_thinking_config};
use loopal_provider_api::{ChatParams, StreamChunk};
use tracing::{error, info, warn};

use super::llm_result::LlmStreamResult;
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

        // Resolve thinking config: Auto → concrete config based on model capability
        let capability = get_thinking_capability(&self.params.model);
        let resolved_thinking = resolve_thinking_config(
            &self.thinking_config, capability, self.max_output_tokens,
        );

        Ok(ChatParams {
            model: self.params.model.clone(),
            messages: messages.to_vec(),
            system_prompt: full_system_prompt,
            tools: tool_defs,
            max_tokens: self.max_output_tokens,
            temperature: None,
            thinking: resolved_thinking,
            debug_dump_dir: Some(loopal_config::tmp_dir()),
        })
    }

    /// Stream the LLM response using a provided working copy of messages.
    pub async fn stream_llm_with(
        &mut self,
        messages: &[Message],
    ) -> Result<LlmStreamResult> {
        let chat_params = self.prepare_chat_params_with(messages)?;
        let provider = self.params.kernel.resolve_provider(&self.params.model)?;

        let llm_start = Instant::now();
        info!(
            model = %self.params.model,
            messages = messages.len(),
            tools = chat_params.tools.len(),
            max_tokens = chat_params.max_tokens,
            thinking = ?chat_params.thinking,
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

        let mut result = LlmStreamResult {
            assistant_text: String::new(),
            tool_uses: Vec::new(),
            stream_error: false,
            stop_reason: loopal_provider_api::StopReason::EndTurn,
            thinking_text: String::new(),
            thinking_signature: None,
            thinking_tokens: 0,
        };

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(StreamChunk::Text { text }) => {
                    result.assistant_text.push_str(&text);
                    self.emit(AgentEventPayload::Stream { text }).await?;
                }
                Ok(StreamChunk::Thinking { text }) => {
                    result.thinking_text.push_str(&text);
                    self.emit(AgentEventPayload::ThinkingStream {
                        text,
                    }).await?;
                }
                Ok(StreamChunk::ThinkingSignature { signature }) => {
                    result.thinking_signature = Some(signature);
                }
                Ok(StreamChunk::ToolUse { id, name, input }) => {
                    self.emit(AgentEventPayload::ToolCall {
                        id: id.clone(), name: name.clone(), input: input.clone(),
                    }).await?;
                    result.tool_uses.push((id, name, input));
                }
                Ok(StreamChunk::Usage {
                    input_tokens, output_tokens,
                    cache_creation_input_tokens, cache_read_input_tokens,
                    thinking_tokens,
                }) => {
                    self.total_input_tokens += input_tokens;
                    self.total_output_tokens += output_tokens;
                    self.total_cache_creation_tokens += cache_creation_input_tokens;
                    self.total_cache_read_tokens += cache_read_input_tokens;
                    self.total_thinking_tokens += thinking_tokens;
                    result.thinking_tokens += thinking_tokens;
                    self.emit(AgentEventPayload::TokenUsage {
                        input_tokens, output_tokens,
                        context_window: self.max_context_tokens,
                        cache_creation_input_tokens, cache_read_input_tokens,
                        thinking_tokens,
                    }).await?;
                }
                Ok(StreamChunk::Done { stop_reason }) => {
                    result.stop_reason = stop_reason;
                    break;
                }
                Err(e) => {
                    error!(error = %e, turn = self.turn_count, model = %self.params.model, "stream error");
                    self.emit(AgentEventPayload::Error { message: e.to_string() }).await?;
                    result.stream_error = true;
                    break;
                }
            }
        }

        // Emit ThinkingComplete after stream ends (Protected Variations:
        // different providers report thinking tokens at different times).
        // Also emit when thinking_tokens > 0 but no thinking_text (e.g. OpenAI
        // reasoning models report tokens in Usage but don't stream content).
        if !result.thinking_text.is_empty() || result.thinking_tokens > 0 {
            let token_count = if result.thinking_text.is_empty() {
                result.thinking_tokens
            } else {
                result.thinking_tokens.max(
                    result.thinking_text.len() as u32 / 4, // fallback estimate
                )
            };
            self.emit(AgentEventPayload::ThinkingComplete { token_count }).await?;
        }

        let llm_duration = llm_start.elapsed();
        info!(
            duration_ms = llm_duration.as_millis() as u64,
            tool_calls = result.tool_uses.len(),
            has_text = !result.assistant_text.is_empty(),
            thinking_tokens = result.thinking_tokens,
            "LLM complete"
        );

        Ok(result)
    }
}
