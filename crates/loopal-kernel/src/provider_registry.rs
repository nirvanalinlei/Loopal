use std::sync::Arc;

use loopal_config::Settings;
use loopal_provider::{
    AnthropicProvider, GoogleProvider, OpenAiCompatProvider, OpenAiProvider, ProviderRegistry,
};
use tracing::info;

pub fn register_providers(settings: &Settings, registry: &mut ProviderRegistry) {
    loopal_provider::init_user_models(&settings.models);
    let providers = &settings.providers;

    let anthropic_key = providers
        .anthropic
        .as_ref()
        .and_then(|c| resolve_api_key(&c.api_key, &c.api_key_env))
        .or_else(|| first_env(&["ANTHROPIC_API_KEY", "OPUS_API_KEY", "ANTHROPIC_AUTH_TOKEN"]));

    if let Some(api_key) = anthropic_key {
        let mut provider = AnthropicProvider::new(api_key.clone());
        let anthropic_base_url = providers
            .anthropic
            .as_ref()
            .and_then(|c| c.base_url.clone())
            .or_else(|| first_env(&["ANTHROPIC_BASE_URL", "OPUS_API_URL", "OPUS_BASE_URL"]));
        if let Some(url) = anthropic_base_url.clone() {
            provider = provider.with_base_url(url);
        }
        let compatibility_bearer = first_env(&["OPUS_AUTH_TOKEN", "ANTHROPIC_AUTH_TOKEN"]).or_else(|| {
            if anthropic_base_url.as_deref().is_some_and(|url| !url.contains("api.anthropic.com"))
                || std::env::var("OPUS_API_URL").ok().is_some()
            {
                Some(api_key.clone())
            } else {
                None
            }
        });
        if let Some(token) = compatibility_bearer {
            provider = provider.with_authorization_bearer(token);
        }
        if let Some(version) = first_env(&["ANTHROPIC_API_VERSION", "OPUS_API_VERSION"]) {
            provider = provider.with_anthropic_version(version);
        }
        if let Some(user_agent) = first_env(&["ANTHROPIC_USER_AGENT", "OPUS_API_USER_AGENT"]) {
            provider = provider.with_user_agent(user_agent);
        }
        if let Some(raw_headers) = first_env(&["ANTHROPIC_EXTRA_HEADERS", "OPUS_EXTRA_HEADERS"]) {
            for (name, value) in parse_extra_headers(&raw_headers) {
                provider = provider.with_extra_header(name, value);
            }
        }
        registry.register(Arc::new(provider));
        info!("registered anthropic provider");
    }

    let openai_key = providers
        .openai
        .as_ref()
        .and_then(|c| resolve_api_key(&c.api_key, &c.api_key_env))
        .or_else(|| std::env::var("OPENAI_API_KEY").ok().filter(|k| !k.is_empty()));

    if let Some(api_key) = openai_key {
        let mut provider = OpenAiProvider::new(api_key);
        if let Some(ref config) = providers.openai
            && let Some(ref base_url) = config.base_url
        {
            provider = provider.with_base_url(base_url.clone());
        }
        registry.register(Arc::new(provider));
        info!("registered openai provider");
    }

    let google_key = providers
        .google
        .as_ref()
        .and_then(|c| resolve_api_key(&c.api_key, &c.api_key_env))
        .or_else(|| std::env::var("GOOGLE_API_KEY").ok().filter(|k| !k.is_empty()));

    if let Some(api_key) = google_key {
        let mut provider = GoogleProvider::new(api_key);
        if let Some(ref config) = providers.google
            && let Some(ref base_url) = config.base_url
        {
            provider = provider.with_base_url(base_url.clone());
        }
        registry.register(Arc::new(provider));
        info!("registered google provider");
    }

    for compat in &providers.openai_compat {
        if let Some(api_key) = resolve_api_key(&compat.api_key, &compat.api_key_env) {
            let provider =
                OpenAiCompatProvider::new(api_key, compat.base_url.clone(), compat.name.clone());
            if let Some(ref prefix) = compat.model_prefix {
                registry.register_with_prefix(Arc::new(provider), prefix);
                info!(name = %compat.name, prefix, "registered openai-compat provider (prefix)");
            } else {
                registry.register(Arc::new(provider));
                info!(name = %compat.name, "registered openai-compat provider");
            }
        }
    }
}

pub fn resolve_api_key(api_key: &Option<String>, api_key_env: &Option<String>) -> Option<String> {
    if let Some(key) = api_key
        && !key.is_empty()
    {
        return Some(key.clone());
    }
    if let Some(env_var) = api_key_env
        && let Ok(key) = std::env::var(env_var)
        && !key.is_empty()
    {
        return Some(key);
    }
    None
}

fn first_env(names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| std::env::var(name).ok().filter(|v| !v.is_empty()))
}

fn parse_extra_headers(raw: &str) -> Vec<(String, String)> {
    raw.split(['\n', ';'])
        .filter_map(|entry| {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                return None;
            }
            let (name, value) = trimmed
                .split_once('=')
                .or_else(|| trimmed.split_once(':'))?;
            let name = name.trim();
            let value = value.trim();
            if name.is_empty() || value.is_empty() {
                return None;
            }
            Some((name.to_string(), value.to_string()))
        })
        .collect()
}
