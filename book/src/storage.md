# 存储后端

Ducia 的存储层通过 `DocRepository` trait 抽象，支持即插即用的后端切换。框架内置两种实现，也可自行开发自定义存储后端。

## 架构概览

```text
HTTP Handler
    │
    ▼
PluginRegistry
    │
    ▼
StoragePlugin (DocRepository trait)
    ├── storage-fs ──→ data/docs.json + docs/*.md
    ├── storage-sqlite ──→ data/ducia.db
    └── 自定义后端
```

所有 HTTP handler 通过 `PluginRegistry` 获取存储插件，调用 `DocRepository` trait 定义的方法。框架不感知底层是文件系统还是数据库。

---

## `DocRepository` trait

定义在 `ducia-core/doc/repo.rs`，所有存储后端必须实现此接口：

```rust
#[async_trait]
pub trait DocRepository: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;

    /// 列出所有文档（可按需过滤已删除文档）
    async fn list_docs(&self, include_deleted: bool) -> Result<Vec<DocMeta>>;

    /// 获取单个文档完整内容
    async fn get_doc(&self, id: &str) -> Result<Option<DocFull>>;

    /// 创建文档
    async fn create_doc(&self, req: CreateDocRequest) -> Result<DocMeta>;

    /// 更新文档元数据（弃用、删除标记）
    async fn update_meta(
        &self,
        id: &str,
        deprecated: Option<bool>,
        deleted: Option<bool>,
    ) -> Result<()>;

    /// 获取站点名称
    async fn site_name(&self) -> String;
}
```

### 核心数据类型

| 类型 | 说明 | 关键字段 |
|------|------|----------|
| `DocMeta` | 文档元数据 | `id`, `title`, `file`, `created_at`, `deprecated`, `deleted` |
| `DocFull` | 文档完整数据 | `id`, `title`, `content`, `created_at`, `deprecated` |
| `CreateDocRequest` | 创建文档请求 | `title`, `content` |

---

## storage-fs — 文件系统存储

**默认后端**，零依赖，适合单机小规模部署。

### 工作原理

```
data/docs.json             ← JSON 格式的文档元数据索引
docs/1.md                 ← 自增ID命名的 Markdown 文件
docs/2.md
```

- 元数据索引存储在 `data/docs.json`（Docker 部署时 `config/` 为只读挂载）
- 文档内容以 `.md` 文件形式存储在 `docs/` 目录
- 每次读写都加载/保存整个索引文件（简单但非原子操作）

### 优点

- **零外部依赖**：仅使用 Rust 标准库
- **人类可读**：所有数据都是纯文本 JSON 和 Markdown
- **易于备份**：直接拷贝 `config/` 和 `docs/` 目录
- **Git 友好**：可将文档内容纳入版本管理

### 缺点

- 非原子操作，并发写入可能数据不一致
- 每次列表操作需解析整个 JSON 文件
- 不支持复杂查询（搜索、排序）

### 关键实现

```rust
pub struct FsStorage {
    config_dir: PathBuf,  // 只读配置目录（site.json）
    data_dir: PathBuf,    // 读写数据目录（docs.json）
    docs_dir: PathBuf,    // 文档内容目录（*.md）
}
```

**`list_docs`** 逻辑：加载 `docs.json` → 过滤 `deprecated=false` 且 `deleted=false` 的文档。

**`create_doc`** 逻辑：自增 `next_id` → 写入 `docs/{id}.md` → 更新 `docs.json` 索引。

---

## storage-sqlite — SQLite 数据库存储

生产环境推荐后端，提供数据库级别的可靠性和查询能力。

### 工作原理

```
data/ducia.db              ← SQLite 数据库文件
  ├── docs 表               ← 文档元数据
  └── settings 表           ← 键值对设置（如 site_name）
docs/1.md                   ← 文档内容仍以文件形式存储
docs/2.md
```

### 数据库 Schema

**`docs` 表**

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `id` | `INTEGER` | `PRIMARY KEY` | 自增文档 ID |
| `title` | `TEXT` | `NOT NULL` | 文档标题 |
| `file` | `TEXT` | `NOT NULL` | 内容文件路径 |
| `created_at` | `INTEGER` | `NOT NULL` | 创建时间戳（毫秒） |
| `deprecated` | `INTEGER` | `DEFAULT 0` | 是否弃用（0/1） |
| `deleted` | `INTEGER` | `DEFAULT 0` | 是否删除（0/1） |

**`settings` 表**

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `key` | `TEXT` | `PRIMARY KEY` | 设置键 |
| `value` | `TEXT` | `NOT NULL` | 设置值 |

### 优点

- **原子操作**：SQLite 事务保证写入一致性
- **SQL 查询**：支持搜索、排序、分页
- **并发安全**：通过 `Mutex<Connection>` 保证线程安全
- **零配置**：SQLite 嵌入式数据库，无需独立服务

### 缺点

- 文档内容仍以文件形式存储（非纯数据库方案）
- 仅限单机部署（SQLite 不是客户端-服务器架构）

### 关键实现

使用 `rusqlite` crate（带 `bundled` feature，自动编译 SQLite C 库）。

```rust
pub struct SqliteStorage {
    conn: Mutex<Connection>,  // 线程安全的数据库连接
    docs_dir: PathBuf,        // 文档内容目录
}
```

初始化时自动建表：

```sql
CREATE TABLE IF NOT EXISTS docs (
    id          INTEGER PRIMARY KEY,
    title       TEXT NOT NULL,
    file        TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    deprecated  INTEGER NOT NULL DEFAULT 0,
    deleted     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

---

## 切换存储后端

### 方法一：修改 `config/settings.json`

将 `use_database` 设置为 `true`：

```json
{
  "show_deprecated": false,
  "show_deleted": false,
  "use_database": true
}
```

重启服务即可。框架在 `main.rs` 中根据此配置选择插件：

```rust
let storage_plugin = if use_db {
    Box::new(SqliteStorage::new(db_path, docs_dir)?) as Box<dyn DocRepository>
} else {
    Box::new(FsStorage::new(config_dir, data_dir, docs_dir))
};
```

### 方法二：从文件系统迁移到 SQLite

如果已有文件系统存储的数据，迁移步骤：

1. 确保 `data/docs.json` 和 `docs/*.md` 数据完好
2. 设置 `use_database: true`
3. 重启服务——新创建的文档将存入 SQLite
4. 旧数据（`data/docs.json`）不会被自动导入，需要手动迁移或通过 API 重新上传

---

## 编写自定义存储后端

实现自定义存储只需要三步：

### 第一步：实现 `DocRepository` trait

```rust
use async_trait::async_trait;
use ducia_core::doc::model::{CreateDocRequest, DocFull, DocMeta};
use ducia_core::doc::repo::DocRepository;

pub struct MyStorage {
    // 你的连接、客户端等
}

#[async_trait]
impl DocRepository for MyStorage {
    fn name(&self) -> &str {
        "my-storage"
    }

    async fn list_docs(&self, include_deleted: bool) -> anyhow::Result<Vec<DocMeta>> {
        // 实现文档列表查询
        todo!()
    }

    async fn get_doc(&self, id: &str) -> anyhow::Result<Option<DocFull>> {
        // 实现单个文档查询
        todo!()
    }

    async fn create_doc(&self, req: CreateDocRequest) -> anyhow::Result<DocMeta> {
        // 实现文档创建
        todo!()
    }

    async fn update_meta(
        &self,
        id: &str,
        deprecated: Option<bool>,
        deleted: Option<bool>,
    ) -> anyhow::Result<()> {
        // 实现元数据更新
        todo!()
    }

    async fn site_name(&self) -> String {
        // 实现站点名获取
        "My Site".into()
    }
}
```

### 第二步：在 `main.rs` 中注册

在 `backend/server/src/main.rs` 中添加你的存储后端分支：

```rust
use my_storage::MyStorage;

// 根据配置或环境变量选择
let storage_plugin = Box::new(MyStorage::new(...)) as Box<dyn DocRepository>;
```

### 第三步（可选）：添加配置项

可在 `settings.json` 中添加自定义字段来控制后端选择：

```json
{
  "storage_backend": "my-storage",
  "my_storage_url": "postgres://localhost/ducia"
}
```

---

## 存储后端对比

| 特性 | storage-fs | storage-sqlite | 自定义 |
|------|------------|---------------|--------|
| 复杂度 | 极简 | 中等 | 按需 |
| 依赖 | 无 | rusqlite | 按需 |
| 原子性 | ❌ | ✅ | 取决于实现 |
| SQL 查询 | ❌ | ✅ | 取决于实现 |
| 并发安全 | ❌ (非原子) | ✅ (Mutex) | 取决于实现 |
| 数据可读性 | ✅ (JSON+MD) | ✅ (SQLite CLI) | 取决于实现 |
| 适合场景 | 开发/小规模 | 生产/中等规模 | 特殊需求 |

### 未来可能的存储后端

得益于 trait 抽象，以下后端可以轻松接入：

| 后端 | 适用场景 |
|------|----------|
| **PostgreSQL** | 大规模部署、高并发 |
| **S3 / MinIO** | 云原生、对象存储 |
| **Redis** | 高性能缓存层 |
| **MongoDB** | 文档型数据、灵活 Schema |
| **Git** | 版本管理的文档仓库 |
