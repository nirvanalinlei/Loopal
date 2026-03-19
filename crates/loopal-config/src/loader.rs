use std::path::Path;

use crate::settings::Settings;
use loopal_error::{ConfigError, LoopalError};

use crate::locations;

/// Deep-merge two JSON values. Objects are merged recursively; all other types
/// (including arrays) are replaced by the overlay value.
pub fn deep_merge(base: &mut serde_json::Value, overlay: serde_json::Value) {
    match (base, overlay) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(overlay_map)) => {
            for (key, value) in overlay_map {
                deep_merge(base_map.entry(key).or_insert(serde_json::Value::Null), value);
            }
        }
        (base, overlay) => {
            *base = overlay;
        }
    }
}

/// Load a JSON file and return its Value, or Null if the file does not exist.
pub fn load_json_file(path: &Path) -> Result<serde_json::Value, LoopalError> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let value: serde_json::Value = serde_json::from_str(&contents)
                .map_err(|e| ConfigError::Parse(format!("{}: {}", path.display(), e)))?;
            Ok(value)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::Value::Null),
        Err(e) => Err(LoopalError::Io(e)),
    }
}

/// Apply environment variable overrides to a JSON value.
pub fn apply_env_overrides(value: &mut serde_json::Value) {
    // Ensure we have an object to work with
    if !value.is_object() {
        *value = serde_json::json!({});
    }

    if let Ok(model) = std::env::var("LOOPAL_MODEL") {
        value["model"] = serde_json::Value::String(model);
    }

    if let Ok(max_turns) = std::env::var("LOOPAL_MAX_TURNS")
        && let Ok(n) = max_turns.parse::<u32>() {
            value["max_turns"] = serde_json::json!(n);
        }

    if let Ok(mode) = std::env::var("LOOPAL_PERMISSION_MODE") {
        value["permission_mode"] = serde_json::Value::String(mode);
    }

    if let Ok(sandbox) = std::env::var("LOOPAL_SANDBOX") {
        value["sandbox"]["policy"] = serde_json::Value::String(sandbox);
    }
}

/// Load settings with 5-layer merge:
/// 1. Defaults (from Settings::default())
/// 2. Global settings.json
/// 3. Project settings.json
/// 4. Project settings.local.json
/// 5. Environment variable overrides
pub fn load_settings(cwd: &Path) -> Result<Settings, LoopalError> {
    // Start with defaults serialized to Value
    let mut merged = serde_json::to_value(Settings::default())
        .map_err(|e| ConfigError::Parse(e.to_string()))?;

    // Layer 2: global settings
    let global = load_json_file(&locations::global_settings_path()?)?;
    if !global.is_null() {
        deep_merge(&mut merged, global);
    }

    // Layer 3: project settings
    let project = load_json_file(&locations::project_settings_path(cwd))?;
    if !project.is_null() {
        deep_merge(&mut merged, project);
    }

    // Layer 4: project local settings
    let local = load_json_file(&locations::project_local_settings_path(cwd))?;
    if !local.is_null() {
        deep_merge(&mut merged, local);
    }

    // Layer 5: environment overrides
    apply_env_overrides(&mut merged);

    // Warn about unrecognised keys before deserialising
    crate::validate::warn_unknown_keys(&merged);

    let settings: Settings = serde_json::from_value(merged)
        .map_err(|e| ConfigError::Parse(e.to_string()))?;

    Ok(settings)
}

/// Load and concatenate instruction files (LOOPAL.md).
/// Global instructions come first, then project instructions, separated by newlines.
pub fn load_instructions(cwd: &Path) -> Result<String, LoopalError> {
    let mut parts = Vec::new();

    let global_path = locations::global_instructions_path()?;
    if global_path.exists() {
        parts.push(std::fs::read_to_string(&global_path)?);
    }

    let project_path = locations::project_instructions_path(cwd);
    if project_path.exists() {
        parts.push(std::fs::read_to_string(&project_path)?);
    }

    Ok(parts.join("\n\n"))
}