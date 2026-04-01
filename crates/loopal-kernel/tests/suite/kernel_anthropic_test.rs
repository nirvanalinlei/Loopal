use loopal_config::{ProviderConfig, ProvidersConfig, Settings};
use loopal_provider::ProviderRegistry;

#[test]
fn test_register_providers_with_config_api_key() {
    let settings = Settings {
        providers: ProvidersConfig {
            anthropic: Some(ProviderConfig {
                api_key: Some("direct-config-key".to_string()),
                api_key_env: None,
                base_url: None,
            }),
            openai: None,
            google: None,
            openai_compat: vec![],
        },
        ..Default::default()
    };

    let mut registry = ProviderRegistry::new();
    loopal_kernel::register_providers(&settings, &mut registry);
    assert!(
        registry.get("anthropic").is_some(),
        "anthropic provider should be registered with direct config api_key"
    );
}

#[test]
fn test_register_providers_with_config_env_key() {
    let env_var = "LOOPAL_TEST_ANTHROPIC_KEY_UNIQUE_92381";
    unsafe {
        std::env::set_var(env_var, "test-key-from-env");
    }

    let settings = Settings {
        providers: ProvidersConfig {
            anthropic: Some(ProviderConfig {
                api_key: None,
                api_key_env: Some(env_var.to_string()),
                base_url: None,
            }),
            openai: None,
            google: None,
            openai_compat: vec![],
        },
        ..Default::default()
    };

    let mut registry = ProviderRegistry::new();
    loopal_kernel::register_providers(&settings, &mut registry);
    assert!(
        registry.get("anthropic").is_some(),
        "anthropic provider should be registered via api_key_env"
    );

    unsafe {
        std::env::remove_var(env_var);
    }
}

#[test]
fn test_register_providers_anthropic_with_base_url() {
    let settings = Settings {
        providers: ProvidersConfig {
            anthropic: Some(ProviderConfig {
                api_key: Some("test-anthro-key-base-url".to_string()),
                api_key_env: None,
                base_url: Some("https://custom-anthropic.example.com".to_string()),
            }),
            openai: None,
            google: None,
            openai_compat: vec![],
        },
        ..Default::default()
    };

    let mut registry = ProviderRegistry::new();
    loopal_kernel::register_providers(&settings, &mut registry);
    assert!(
        registry.get("anthropic").is_some(),
        "anthropic provider should be registered with base_url"
    );
}

#[test]
fn test_register_providers_anthropic_from_auth_token_env() {
    let auth_token_env = "ANTHROPIC_AUTH_TOKEN";
    let api_key_env = "ANTHROPIC_API_KEY";
    let orig_api_key = std::env::var(api_key_env).ok();
    let orig_auth_token = std::env::var(auth_token_env).ok();

    unsafe {
        std::env::remove_var(api_key_env);
        std::env::set_var(auth_token_env, "test-auth-token-fallback-value");
    }

    let settings = Settings {
        providers: ProvidersConfig {
            anthropic: None,
            openai: None,
            google: None,
            openai_compat: vec![],
        },
        ..Default::default()
    };

    let mut registry = ProviderRegistry::new();
    loopal_kernel::register_providers(&settings, &mut registry);
    assert!(
        registry.get("anthropic").is_some(),
        "anthropic should be registered via ANTHROPIC_AUTH_TOKEN fallback"
    );

    unsafe {
        std::env::remove_var(auth_token_env);
        match orig_api_key {
            Some(v) => std::env::set_var(api_key_env, v),
            None => std::env::remove_var(api_key_env),
        }
        match orig_auth_token {
            Some(v) => std::env::set_var(auth_token_env, v),
            None => std::env::remove_var(auth_token_env),
        }
    }
}

#[test]
fn test_register_providers_anthropic_config_with_empty_api_key() {
    let result = loopal_kernel::resolve_api_key(
        &Some(String::new()),
        &Some("NONEXISTENT_ANTHRO_KEY_VAR_99999".to_string()),
    );
    assert_eq!(
        result, None,
        "empty key + nonexistent env var should return None"
    );
}

#[test]
fn test_register_providers_anthropic_env_base_url() {
    let api_key_env = "ANTHROPIC_API_KEY";
    let base_url_env = "ANTHROPIC_BASE_URL";
    let orig_api_key = std::env::var(api_key_env).ok();
    let orig_base_url = std::env::var(base_url_env).ok();

    unsafe {
        std::env::set_var(api_key_env, "test-env-api-key-for-base-url");
        std::env::set_var(base_url_env, "https://env-base-url.example.com");
    }

    let settings = Settings {
        providers: ProvidersConfig {
            anthropic: None,
            openai: None,
            google: None,
            openai_compat: vec![],
        },
        ..Default::default()
    };

    let mut registry = ProviderRegistry::new();
    loopal_kernel::register_providers(&settings, &mut registry);
    assert!(
        registry.get("anthropic").is_some(),
        "anthropic should be registered via ANTHROPIC_API_KEY + ANTHROPIC_BASE_URL env vars"
    );

    unsafe {
        match orig_api_key {
            Some(v) => std::env::set_var(api_key_env, v),
            None => std::env::remove_var(api_key_env),
        }
        match orig_base_url {
            Some(v) => std::env::set_var(base_url_env, v),
            None => std::env::remove_var(base_url_env),
        }
    }
}

#[test]
fn test_register_providers_anthropic_from_opus_env() {
    let orig_opus_key = std::env::var("OPUS_API_KEY").ok();
    let orig_opus_url = std::env::var("OPUS_API_URL").ok();
    let orig_anthropic_key = std::env::var("ANTHROPIC_API_KEY").ok();
    let orig_anthropic_url = std::env::var("ANTHROPIC_BASE_URL").ok();

    unsafe {
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("ANTHROPIC_BASE_URL");
        std::env::set_var("OPUS_API_KEY", "test-opus-key");
        std::env::set_var("OPUS_API_URL", "http://localhost:8080/v1/messages");
    }

    let settings = Settings {
        providers: ProvidersConfig {
            anthropic: None,
            openai: None,
            google: None,
            openai_compat: vec![],
        },
        ..Default::default()
    };

    let mut registry = ProviderRegistry::new();
    loopal_kernel::register_providers(&settings, &mut registry);
    assert!(
        registry.get("anthropic").is_some(),
        "anthropic should be registered via OPUS_API_KEY + OPUS_API_URL env vars"
    );

    unsafe {
        match orig_opus_key {
            Some(v) => std::env::set_var("OPUS_API_KEY", v),
            None => std::env::remove_var("OPUS_API_KEY"),
        }
        match orig_opus_url {
            Some(v) => std::env::set_var("OPUS_API_URL", v),
            None => std::env::remove_var("OPUS_API_URL"),
        }
        match orig_anthropic_key {
            Some(v) => std::env::set_var("ANTHROPIC_API_KEY", v),
            None => std::env::remove_var("ANTHROPIC_API_KEY"),
        }
        match orig_anthropic_url {
            Some(v) => std::env::set_var("ANTHROPIC_BASE_URL", v),
            None => std::env::remove_var("ANTHROPIC_BASE_URL"),
        }
    }
}
