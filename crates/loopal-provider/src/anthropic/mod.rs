mod accumulator;
mod request;
pub(crate) mod server_tool;
mod stream;
mod stream_parser;
mod thinking;

use async_trait::async_trait;
use loopal_error::{LoopalError, ProviderError};
use loopal_provider_api::{ChatParams, ChatStream, Provider};
use reqwest::Client;
use serde_json::json;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::sync::Semaphore;

use crate::sse::SseStream;
use stream::{ServerToolAccumulator, ThinkingAccumulator, ToolUseAccumulator};

const MAX_CONCURRENT_REQUESTS: usize = 3;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
    authorization_bearer: Option<String>,
    anthropic_version: String,
    user_agent: Option<String>,
    extra_headers: Vec<(String, String)>,
    request_semaphore: Semaphore,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            authorization_bearer: None,
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
            anthropic_version: "2023-06-01".to_string(),
            user_agent: None,
            extra_headers: Vec::new(),
            request_semaphore: Semaphore::new(MAX_CONCURRENT_REQUESTS),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn with_authorization_bearer(mut self, token: String) -> Self {
        if !token.is_empty() {
            self.authorization_bearer = Some(token);
        }
        self
    }

    pub fn with_anthropic_version(mut self, version: String) -> Self {
        if !version.is_empty() {
            self.anthropic_version = version;
        }
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        if !user_agent.is_empty() {
            self.user_agent = Some(user_agent);
        }
        self
    }

    pub fn with_extra_header(mut self, name: String, value: String) -> Self {
        if !name.is_empty() && !value.is_empty() {
            self.extra_headers.push((name, value));
        }
        self
    }

    fn messages_endpoint(&self) -> String {
        let trimmed = self.base_url.trim_end_matches('/');
        if trimmed.ends_with("/v1/messages") {
            trimmed.to_string()
        } else if trimmed.ends_with("/v1") {
            format!("{trimmed}/messages")
        } else {
            format!("{trimmed}/v1/messages")
        }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn stream_chat(&self, params: &ChatParams) -> Result<ChatStream, LoopalError> {
        let _permit = self
            .request_semaphore
            .acquire()
            .await
            .map_err(|_| ProviderError::Http("request semaphore closed".into()))?;
        self.do_stream_chat(params).await
    }
}

impl AnthropicProvider {
    async fn do_stream_chat(&self, params: &ChatParams) -> Result<ChatStream, LoopalError> {
        let normalized = loopal_message::normalize_messages(&params.messages);
        let normalized_params = ChatParams {
            messages: normalized,
            ..params.clone()
        };
        let messages = self.build_messages(&normalized_params);
        let tools = self.build_tools(params);
        let mut body = json!({
            "model": params.model,
            "max_tokens": params.max_tokens,
            "stream": true,
            "messages": messages,
        });

        if !params.system_prompt.is_empty() {
            body["system"] = json!([{
                "type": "text",
                "text": params.system_prompt,
                "cache_control": {"type": "ephemeral"}
            }]);
        }
        if !tools.is_empty() {
            body["tools"] = json!(tools);
        }
        if let Some(temp) = params.temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(ref thinking_config) = params.thinking {
            body["thinking"] = thinking::to_anthropic_thinking(thinking_config, params.max_tokens);
            if let Some(output_config) = thinking::to_anthropic_output_config(thinking_config) {
                body["output_config"] = output_config;
            }
        }

        let url = self.messages_endpoint();
        tracing::info!(
            model = %params.model, url = %url,
            messages = params.messages.len(), tools = params.tools.len(),
            max_tokens = params.max_tokens,
            body_bytes = body.to_string().len(),
            "API request"
        );

        let mut request = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.anthropic_version)
            .header("content-type", "application/json");
        if let Some(token) = &self.authorization_bearer {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        if let Some(user_agent) = &self.user_agent {
            request = request.header("user-agent", user_agent);
        }
        for (name, value) in &self.extra_headers {
            request = request.header(name.as_str(), value.as_str());
        }

        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;

        let status = response.status();
        tracing::info!(status = status.as_u16(), "API response");
        if !status.is_success() {
            if let Some(ref dump_dir) = params.debug_dump_dir {
                let _ = std::fs::create_dir_all(dump_dir);
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0);
                let path = dump_dir.join(format!("api_error_{status}_{ts}.json"));
                let _ = std::fs::write(&path, body.to_string());
                tracing::warn!(path = %path.display(), "dumped failed request body");
            }
            return Err(self.handle_error_response(response, status).await);
        }

        let sse = SseStream::from_response(response);
        let stream = stream::AnthropicStream {
            inner: Box::pin(sse),
            tool_state: ToolUseAccumulator::default(),
            thinking_state: ThinkingAccumulator::default(),
            server_tool_state: ServerToolAccumulator::default(),
            buffer: VecDeque::new(),
        };
        Ok(Box::pin(stream))
    }

    async fn handle_error_response(
        &self,
        response: reqwest::Response,
        status: reqwest::StatusCode,
    ) -> LoopalError {
        if status.as_u16() == 429 {
            let retry_after_ms = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|secs| (secs * 1000.0) as u64)
                .unwrap_or(30_000);
            tracing::warn!(retry_after_ms, "rate limited by API");
            return ProviderError::RateLimited { retry_after_ms }.into();
        }
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to read body".into());
        tracing::error!(status = status.as_u16(), body = %text, "API error");
        if status.as_u16() == 400
            && (text.contains("prompt is too long")
                || text.contains("maximum context length")
                || text.contains("invalid_request_error"))
        {
            return ProviderError::ContextOverflow { message: text }.into();
        }
        ProviderError::Api {
            status: status.as_u16(),
            message: text,
        }
        .into()
    }
}
