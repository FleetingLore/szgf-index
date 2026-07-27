//! Utilities to load site configuration values from files in a config directory.
//!
//! These helpers read JSON files such as `site.json` and `sequence.json`.
use std::fs;
use std::path::PathBuf;

/// Load the site name from `site.json` located in `config_dir`.
///
/// Returns the `name` field if present; otherwise returns a default site name.
pub fn load_site_name(config_dir: &PathBuf) -> String {
    let path = config_dir.join("site.json");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                return name.to_string();
            }
        }
    }
    "25 电科 3 班待办事项清单".to_string()
}

/// Load the admin sequence from `sequence.json` in `config_dir`.
///
/// Returns a vector of `usize` values. If the file is missing or malformed,
/// an empty vector is returned.
pub fn load_admin_sequence(config_dir: &PathBuf) -> Vec<usize> {
    let path = config_dir.join("sequence.json");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(arr) = json.get("sequence").and_then(|s| s.as_array()) {
                let seq: Vec<usize> = arr
                    .iter()
                    .filter_map(|v| v.as_u64())
                    .map(|v| v as usize)
                    .collect();
                if !seq.is_empty() {
                    return seq;
                }
            }
        }
    }
    // 如果文件不存在或格式错误，返回空数组
    Vec::new()
}
