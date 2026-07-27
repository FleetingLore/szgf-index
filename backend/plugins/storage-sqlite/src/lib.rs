//! storage-sqlite — SQLite 存储插件
//!
//! 使用 SQLite 数据库存储所有文档和元数据。
//! 相比文件系统存储，提供了：
//! - 原子性操作
//! - SQL 查询（搜索、排序、分页）
//! - 并发安全（连接池）
//! - 结构化数据管理

use async_trait::async_trait;
use ducia_core::doc::model::{CreateDocRequest, DocFull, DocMeta};
use ducia_core::doc::repo::DocRepository;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct SqliteStorage {
    conn: Mutex<Connection>,
    docs_dir: PathBuf,
}

impl SqliteStorage {
    /// 打开或创建 SQLite 数据库
    pub fn new(db_path: PathBuf, docs_dir: PathBuf) -> anyhow::Result<Self> {
        let _ = std::fs::create_dir_all(db_path.parent().unwrap_or(&db_path));
        let _ = std::fs::create_dir_all(&docs_dir);

        let conn = Connection::open(&db_path)?;

        // 创建表（如果不存在）
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS docs (
                id          INTEGER PRIMARY KEY,
                title       TEXT NOT NULL,
                file        TEXT NOT NULL,
                created_at  INTEGER NOT NULL,
                deprecated  INTEGER NOT NULL DEFAULT 0,
                deleted     INTEGER NOT NULL DEFAULT 0,
                locked      INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            INSERT OR IGNORE INTO settings (key, value) VALUES ('site_name', 'Ducia Listing');",
        )?;

        // 迁移：已有表但没有 locked 列时添加
        let _ =
            conn.execute_batch("ALTER TABLE docs ADD COLUMN locked INTEGER NOT NULL DEFAULT 0;");

        Ok(Self {
            conn: Mutex::new(conn),
            docs_dir,
        })
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

#[async_trait]
impl DocRepository for SqliteStorage {
    fn name(&self) -> &str {
        "storage-sqlite"
    }

    async fn list_docs(&self, _include_deleted: bool) -> anyhow::Result<Vec<DocMeta>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, file, created_at, deprecated, deleted, locked
             FROM docs WHERE deleted = 0 AND deprecated = 0
             ORDER BY created_at DESC",
        )?;

        let docs = stmt.query_map([], |row| {
            Ok(DocMeta {
                id: row.get::<_, i64>(0)?.to_string(),
                title: row.get(1)?,
                file: row.get(2)?,
                created_at: row.get::<_, i64>(3)? as u64,
                deprecated: row.get::<_, i64>(4)? != 0,
                deleted: row.get::<_, i64>(5)? != 0,
                locked: row.get::<_, i64>(6)? != 0,
            })
        })?;

        Ok(docs.filter_map(|r| r.ok()).collect())
    }

    async fn get_doc(&self, id: &str) -> anyhow::Result<Option<DocFull>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, file, created_at, deprecated, locked
             FROM docs WHERE id = ?1 AND deleted = 0",
        )?;

        let doc = stmt
            .query_row([id.parse::<i64>().unwrap_or(0)], |row| {
                Ok(DocMeta {
                    id: row.get::<_, i64>(0)?.to_string(),
                    title: row.get(1)?,
                    file: row.get(2)?,
                    created_at: row.get::<_, i64>(3)? as u64,
                    deprecated: row.get::<_, i64>(4)? != 0,
                    deleted: false,
                    locked: row.get::<_, i64>(5)? != 0,
                })
            })
            .ok();

        match doc {
            Some(meta) => {
                let content =
                    std::fs::read_to_string(self.docs_dir.join(&meta.file)).unwrap_or_default();
                Ok(Some(DocFull {
                    id: meta.id,
                    title: meta.title,
                    content,
                    created_at: meta.created_at,
                    deprecated: meta.deprecated,
                    locked: meta.locked,
                }))
            }
            None => Ok(None),
        }
    }

    async fn create_doc(&self, req: CreateDocRequest) -> anyhow::Result<DocMeta> {
        let conn = self.conn.lock().unwrap();
        let now = Self::now_ms();
        let filename = format!("{}.md", now);
        let content_path = self.docs_dir.join(&filename);

        std::fs::write(&content_path, &req.content)?;

        conn.execute(
            "INSERT INTO docs (title, file, created_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![req.title, filename, now as i64],
        )?;

        let id = conn.last_insert_rowid();

        Ok(DocMeta {
            id: id.to_string(),
            title: req.title,
            file: filename,
            created_at: now,
            deprecated: false,
            deleted: false,
            locked: false,
        })
    }

    async fn update_meta(
        &self,
        id: &str,
        deprecated: Option<bool>,
        deleted: Option<bool>,
        locked: Option<bool>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let id_num = id.parse::<i64>().unwrap_or(0);

        if let Some(v) = deprecated {
            conn.execute(
                "UPDATE docs SET deprecated = ?1 WHERE id = ?2",
                rusqlite::params![v as i64, id_num],
            )?;
        }
        if let Some(v) = deleted {
            conn.execute(
                "UPDATE docs SET deleted = ?1 WHERE id = ?2",
                rusqlite::params![v as i64, id_num],
            )?;
        }
        if let Some(v) = locked {
            conn.execute(
                "UPDATE docs SET locked = ?1 WHERE id = ?2",
                rusqlite::params![v as i64, id_num],
            )?;
        }
        Ok(())
    }

    async fn site_name(&self) -> String {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT value FROM settings WHERE key = 'site_name'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "Ducia Listing".into())
    }
}
