//! 文档存储 trait
//!
//! 所有存储后端（文件系统、SQLite、S3...）实现此 trait。

use crate::doc::model::{CreateDocRequest, DocFull, DocMeta};
use anyhow::Result;
use async_trait::async_trait;

/// 文档仓库抽象
///
/// 框架通过此 trait 与存储交互，不关心底层实现。
#[async_trait]
pub trait DocRepository: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;

    /// 列出所有文档（可按需过滤）
    async fn list_docs(&self, include_deleted: bool) -> Result<Vec<DocMeta>>;

    /// 获取单个文档完整内容
    async fn get_doc(&self, id: &str) -> Result<Option<DocFull>>;

    /// 创建文档
    async fn create_doc(&self, req: CreateDocRequest) -> Result<DocMeta>;

    /// 更新文档元数据（弃用/删除/锁定标记）
    async fn update_meta(
        &self,
        id: &str,
        deprecated: Option<bool>,
        deleted: Option<bool>,
        locked: Option<bool>,
    ) -> Result<()>;

    /// 获取站点名称
    async fn site_name(&self) -> String;
}
