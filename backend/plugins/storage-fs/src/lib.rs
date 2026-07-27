//! storage-fs — 文件系统存储插件
//!
//! 使用 JSON 文件存储元数据索引，Markdown 文件存储文档内容。
//! 这是最简实现，适合单机部署。可替换为数据库存储插件。

use async_trait::async_trait;
use ducia_core::doc::model::{CreateDocRequest, DocFormat, DocFull, DocMeta};
use ducia_core::doc::repo::DocRepository;
use std::collections::HashMap;
use std::path::PathBuf;

/// 文件系统存储
pub struct FsStorage {
    /// config 目录（只读配置：site.json）
    config_dir: PathBuf,
    /// data 目录（读写数据：docs.json）
    data_dir: PathBuf,
    /// docs 目录（存储 .md 文件）
    docs_dir: PathBuf,
}

impl FsStorage {
    pub fn new(config_dir: PathBuf, data_dir: PathBuf, docs_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&config_dir);
        let _ = std::fs::create_dir_all(&data_dir);
        let _ = std::fs::create_dir_all(&docs_dir);

        // 迁移：如果 data_dir 中没有 docs.json 但 config_dir 中有，则复制
        let data_path = data_dir.join("docs.json");
        let config_path = config_dir.join("docs.json");
        if !data_path.exists() && config_path.exists() {
            let _ = std::fs::copy(&config_path, &data_path);
        }

        Self {
            config_dir,
            data_dir,
            docs_dir,
        }
    }

    /// 加载文档索引
    fn load_docs_map(&self) -> DocsMap {
        let path = self.data_dir.join("docs.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_else(|| DocsMap {
                next_id: 1,
                docs: HashMap::new(),
            })
    }

    /// 保存文档索引
    fn save_docs_map(&self, map: &DocsMap) -> anyhow::Result<()> {
        std::fs::write(
            self.data_dir.join("docs.json"),
            serde_json::to_string_pretty(map)?,
        )?;
        Ok(())
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

/// 文档索引（从 docs.json 反序列化）
#[derive(serde::Serialize, serde::Deserialize)]
struct DocsMap {
    next_id: u64,
    docs: HashMap<String, DocMeta>,
}

#[async_trait]
impl DocRepository for FsStorage {
    fn name(&self) -> &str {
        "storage-fs"
    }

    async fn list_docs(&self, _include_deleted: bool) -> anyhow::Result<Vec<DocMeta>> {
        let docs_map = self.load_docs_map();
        let docs: Vec<DocMeta> = docs_map
            .docs
            .into_iter()
            .filter(|(_, doc)| !doc.deleted && !doc.deprecated)
            .map(|(id, doc)| DocMeta { id, ..doc })
            .collect();
        Ok(docs)
    }

    async fn get_doc(&self, id: &str) -> anyhow::Result<Option<DocFull>> {
        let docs_map = self.load_docs_map();
        let doc = match docs_map.docs.get(id) {
            Some(d) if !d.deleted => d.clone(),
            _ => return Ok(None),
        };

        let content = std::fs::read_to_string(self.docs_dir.join(&doc.file)).unwrap_or_default();

        Ok(Some(DocFull {
            id: id.to_string(),
            title: doc.title,
            content,
            created_at: doc.created_at,
            deprecated: doc.deprecated,
            locked: doc.locked,
        }))
    }

    async fn create_doc(&self, req: CreateDocRequest) -> anyhow::Result<DocMeta> {
        let mut docs_map = self.load_docs_map();
        let new_id = docs_map.next_id;
        let format = DocFormat::Markdown;
        let filename = format!("{}.{}", new_id, format.extension());

        std::fs::write(self.docs_dir.join(&filename), &req.content)?;

        let meta = DocMeta {
            id: new_id.to_string(),
            title: req.title,
            file: filename,
            created_at: Self::now_ms(),
            deprecated: false,
            deleted: false,
            locked: false,
        };

        docs_map.docs.insert(new_id.to_string(), meta.clone());
        docs_map.next_id = new_id + 1;
        self.save_docs_map(&docs_map)?;

        Ok(meta)
    }

    async fn update_meta(
        &self,
        id: &str,
        deprecated: Option<bool>,
        deleted: Option<bool>,
        locked: Option<bool>,
    ) -> anyhow::Result<()> {
        let mut docs_map = self.load_docs_map();
        if let Some(doc) = docs_map.docs.get_mut(id) {
            if let Some(v) = deprecated {
                doc.deprecated = v;
            }
            if let Some(v) = deleted {
                doc.deleted = v;
            }
            if let Some(v) = locked {
                doc.locked = v;
            }
            self.save_docs_map(&docs_map)?;
        }
        Ok(())
    }

    async fn site_name(&self) -> String {
        let path = self.config_dir.join("site.json");
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                    return name.to_string();
                }
            }
        }
        "Ducia Listing".to_string()
    }
}
