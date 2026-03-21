use loopal_config::loader::{apply_env_overrides, deep_merge, load_json_file};
use tempfile::TempDir;

#[test]
fn test_deep_merge_replaces_non_object() {
    let mut base = serde_json::json!("a string");
    let overlay = serde_json::json!("replaced");
    deep_merge(&mut base, overlay);
    assert_eq!(base, serde_json::json!("replaced"));
}

#[test]
fn test_deep_merge_objects_recursive() {
    let mut base = serde_json::json!({"a": {"b": 1, "c": 2}});
    let overlay = serde_json::json!({"a": {"b": 10}});
    deep_merge(&mut base, overlay);
    assert_eq!(base["a"]["b"], 10);
    assert_eq!(base["a"]["c"], 2);
}

#[test]
fn test_deep_merge_object_replaces_non_object_at_key() {
    let mut base = serde_json::json!({"key": "string_value"});
    let overlay = serde_json::json!({"key": {"nested": true}});
    deep_merge(&mut base, overlay);
    assert_eq!(base["key"]["nested"], true);
}

#[test]
fn test_load_json_file_not_found_returns_null() {
    let path = std::path::Path::new("/tmp/loopal_test_nonexistent_file_xyz_12345.json");
    let result = load_json_file(path).unwrap();
    assert!(result.is_null());
}

#[test]
fn test_load_json_file_valid_json() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("test.json");
    std::fs::write(&file, r#"{"key": "value"}"#).unwrap();

    let result = load_json_file(&file).unwrap();
    assert_eq!(result["key"], "value");
}

#[test]
fn test_load_json_file_invalid_json() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("bad.json");
    std::fs::write(&file, "not valid json!").unwrap();

    let result = load_json_file(&file);
    assert!(result.is_err());
}

#[test]
fn test_load_json_file_io_error() {
    let tmp = TempDir::new().unwrap();
    let result = load_json_file(tmp.path());
    assert!(result.is_err());
}

#[test]
fn test_apply_env_overrides_on_non_object() {
    let mut value = serde_json::json!("a string");
    apply_env_overrides(&mut value);
    assert!(value.is_object());
}

#[test]
fn test_apply_env_overrides_on_object() {
    let mut value = serde_json::json!({"existing": true});
    apply_env_overrides(&mut value);
    assert!(value.is_object());
    assert_eq!(value["existing"], true);
}
