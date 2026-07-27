//! Persistence helpers for document metadata and content.
//!
//! Functions here read and write `docs.json` and document markdown files.
use crate::models::DocsMap;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Load the `DocsMap` structure from `docs.json` in `config_dir`.
///
/// If the file is missing or cannot be parsed, a default empty `DocsMap`
/// with `next_id = 1` is returned.
pub fn load_docs_map(config_dir: &PathBuf) -> DocsMap {
    let path = config_dir.join("docs.json");
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_else(|| DocsMap {
            next_id: 1,
            docs: std::collections::HashMap::new(),
        })
}

/// Persist the provided `DocsMap` into `docs.json` under `config_dir`.
pub fn save_docs_map(config_dir: &PathBuf, map: &DocsMap) -> Result<()> {
    fs::write(
        config_dir.join("docs.json"),
        serde_json::to_string_pretty(map)?,
    )?;
    Ok(())
}

/// Read the raw markdown content for `filename` from `docs_dir`.
///
/// If the file cannot be read, a placeholder string is returned.
pub fn get_doc_content(docs_dir: &PathBuf, filename: &str) -> String {
    fs::read_to_string(docs_dir.join(filename)).unwrap_or_else(|_| "# 文档内容丢失".to_string())
}

/// Save raw markdown `content` into `filename` under `docs_dir`.
pub fn save_doc_content(docs_dir: &PathBuf, filename: &str, content: &str) -> Result<()> {
    fs::write(docs_dir.join(filename), content)?;
    Ok(())
}

/// Return the current system time in milliseconds since the UNIX epoch.
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
