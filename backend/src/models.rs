//! Data models and request DTOs used by the backend API.
//!
//! These types are serialized to/from JSON when communicating with clients.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for a single document stored by the service.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocMeta {
    /// Human-readable title of the document.
    pub title: String,
    /// Filename where the document content is stored (relative to the docs directory).
    pub file: String,
    /// Creation timestamp in milliseconds since UNIX epoch.
    pub created_at: u64,
    /// Whether the document is marked as deprecated (soft-deprecated).
    pub deprecated: bool,
    /// Whether the document has been deleted (soft-delete flag).
    pub deleted: bool,
}

/// Container mapping document IDs to metadata and the next available ID.
#[derive(Debug, Serialize, Deserialize)]
pub struct DocsMap {
    /// Next numeric id to assign when creating a new document.
    pub next_id: u64,
    /// Map from document id string to `DocMeta`.
    pub docs: HashMap<String, DocMeta>,
}

/// Request payload for creating a new document.
#[derive(Debug, Deserialize)]
pub struct CreateDocReq {
    /// Document title provided by the client.
    pub title: String,
    /// Raw markdown content for the new document.
    pub content: String,
}

/// Payload used to mark a document as deprecated or not.
#[derive(Debug, Deserialize)]
pub struct DeprecatedReq {
    /// `true` to mark deprecated, `false` to unmark.
    pub deprecated: bool,
}
