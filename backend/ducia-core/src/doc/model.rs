//! 文档领域模型
//!
//! 这些类型是整个框架的数据基础，前端通过 API 与之交互。

use serde::{Deserialize, Serialize};

/// 文档元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMeta {
    /// 文档唯一标识
    pub id: String,
    /// 标题
    pub title: String,
    /// 存储文件名
    pub file: String,
    /// 创建时间戳（ms）
    pub created_at: u64,
    /// 是否弃用
    pub deprecated: bool,
    /// 是否已删除
    pub deleted: bool,
    /// 是否已锁定（锁定后非管理员不可操作）
    #[serde(default)]
    pub locked: bool,
}

/// 文档完整数据（含内容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocFull {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: u64,
    pub deprecated: bool,
    pub locked: bool,
}

/// 创建文档请求
#[derive(Debug, Deserialize)]
pub struct CreateDocRequest {
    pub title: String,
    pub content: String,
}

/// 弃用标记请求
#[derive(Debug, Deserialize)]
pub struct DeprecatedRequest {
    pub deprecated: bool,
}

/// 锁定请求
#[derive(Debug, Deserialize)]
pub struct LockRequest {
    pub locked: bool,
}

/// 支持的文档格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocFormat {
    Markdown,
    PlainText,
    Html,
}

impl DocFormat {
    /// 从文件扩展名推断格式
    pub fn from_filename(name: &str) -> Self {
        match name
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase()
            .as_str()
        {
            "md" | "markdown" => DocFormat::Markdown,
            "txt" => DocFormat::PlainText,
            "html" | "htm" => DocFormat::Html,
            _ => DocFormat::Markdown, // 默认
        }
    }

    /// 格式对应的文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            DocFormat::Markdown => "md",
            DocFormat::PlainText => "txt",
            DocFormat::Html => "html",
        }
    }
}
