# 插件系统

插件系统是 Ducia 的骨架。框架本体只定义 trait 契约，所有业务逻辑通过插件注入。

## 设计原则

- **面向 trait 编程**：框架不 import 任何具体插件实现，只依赖 `ducia-core` 中定义的 trait
- **一对多可替换**：一个 trait 可以有多个实现，部署时按配置选择
- **编译时组合**：插件在 `main.rs` 编译时注册，非运行时动态加载

## 插件架构图

```text
ducia-core (trait 定义)
    ├── AuthPlugin trait ──── → auth-simple (序列码)
    │                       └── auth-db (JWT + 数据库)
    ├── DocRepository trait ── → storage-fs (文件系统)
    │                        └── storage-sqlite (SQLite)
    └── PluginRegistry (组装所有插件)
```

## AuthPlugin trait

定义在 `ducia-core/src/plugin/auth.rs`，是认证机制的抽象接口：

```rust
#[async_trait]
pub trait AuthPlugin: Send + Sync {
    /// 插件唯一名称
    fn name(&self) -> &str;

    /// 从请求头中提取身份，返回 None 表示匿名
    async fn authenticate(&self, headers: &HashMap<String, String>) -> Option<Identity>;

    /// 创建会话，返回 token/凭证
    async fn create_session(&self, identity: &Identity) -> Result<String>;

    /// 销毁会话
    async fn destroy_session(&self, token: &str) -> Result<()>;

    /// 检查会话是否有效
    async fn check_session(&self, token: &str) -> Option<Identity>;
}
```

每个方法的职责：

- `authenticate` — handler 将请求头传入，插件负责从中提取身份。这是**请求入口**，失败返回 `None` 表示匿名用户
- `create_session` — 给定一个 `Identity`，创建持久会话并返回 token。SimpleAuth 生成 UUID 存内存，AuthDb 签发 JWT
- `destroy_session` — 销毁指定会话。SimpleAuth 从 HashMap 移除，JWT 无法服务端销毁（直接返回 `Ok(())`）
- `check_session` — 验证 token 有效性并返回身份

匿名用户通过 `ducia_core::plugin::auth::anonymous()` 函数创建：

```rust
pub fn anonymous() -> Identity {
    Identity {
        id: "anonymous".into(),
        roles: vec!["anonymous".into()],
        permissions: vec!["doc:read".into()],
        metadata: Default::default(),
    }
}
```

## DocRepository trait

定义在 `ducia-core/src/doc/repo.rs`，是文档存储的抽象接口：

```rust
#[async_trait]
pub trait DocRepository: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;

    /// 列出所有文档
    async fn list_docs(&self, include_deleted: bool) -> Result<Vec<DocMeta>>;

    /// 获取单个文档完整内容
    async fn get_doc(&self, id: &str) -> Result<Option<DocFull>>;

    /// 创建文档
    async fn create_doc(&self, req: CreateDocRequest) -> Result<DocMeta>;

    /// 更新文档元数据（弃用/删除标记）
    async fn update_meta(
        &self, id: &str,
        deprecated: Option<bool>,
        deleted: Option<bool>,
    ) -> Result<()>;

    /// 获取站点名称
    async fn site_name(&self) -> String;
}
```

## StoragePlugin 类型别名

存储插件不需要额外的 trait——它直接使用 `DocRepository` 的 trait object：

```rust
pub type StoragePlugin = Box<dyn DocRepository>;
```

## PluginRegistry

`PluginRegistry` 是插件的容器，用 builder 模式构建：

```rust
pub struct PluginRegistry {
    auth: Option<Arc<dyn AuthPlugin>>,
    storage: Option<StoragePlugin>,
    extras: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl PluginRegistry {
    pub fn new() -> Self { /* 空注册表 */ }

    pub fn with_auth(mut self, plugin: Arc<dyn AuthPlugin>) -> Self { /* ... */ }

    pub fn with_storage(mut self, plugin: StoragePlugin) -> Self { /* ... */ }

    pub fn auth(&self) -> Option<&Arc<dyn AuthPlugin>> { /* ... */ }

    pub fn storage(&self) -> Option<&StoragePlugin> { /* ... */ }

    pub fn register_extra<T: 'static + Send + Sync>(&mut self, name: &str, plugin: T) { /* ... */ }
}
```

## 现有插件

### auth-simple（序列码认证）

- 路径：`backend/plugins/auth-simple/`
- 依赖：`ducia-core`、`tokio`、`uuid`
- 原理：前端通过序列码按钮交互验证，后端验证序列后创建内存中的 UUID session
- 配置：`config/sequence.json`（定义验证序列数组）
- 特点：最简单，无需数据库，适合开发调试或单机单用户

**前端交互**：管理面板是一个 2×2 按钮网格，排列为 `[2,1,3,4]`（参见 [数制设计](authentication.html#数制设计)）。每个按钮是翻转开关，由点击次数决定颜色：

| 点击次数 | 视觉效果 |
|----------|----------|
| 0（未点过） | `#1f2328` 边框 + 半透明黑底 |
| 奇数次 | `#90DFDF` 边框 + 半透明青蓝底 |
| 偶数次（>0） | 同未点过 |

颜色由 `clickCounts[num] % 2` 驱动，计数永不清零。序列验证独立于颜色：每次点击将数字追加到 `userInput`，末尾匹配则登录成功，前缀不匹配则清空 `userInput`（但颜色不变——用户需手动 toggle 回暗色或继续点击）。

```rust
pub struct SimpleAuth {
    config_dir: PathBuf,
    sessions: Mutex<HashMap<String, Identity>>,
}
```

### auth-db（数据库认证）

- **路径**：`backend/plugins/auth-db/`
- **依赖**：`ducia-core`、`rusqlite`、`bcrypt`、`jsonwebtoken`、`chrono`
  - `rusqlite` — 操作 SQLite 数据库，存储用户注册信息
  - `bcrypt` — 密码哈希与验证，使用 `DEFAULT_COST=12`
  - `jsonwebtoken` — 签发与验证 JWT token（HS256 算法）
  - `chrono` — 生成 UTC 时间戳（`created_at` 字段和 JWT 的 `iat`/`exp` claims）
- **原理**：用户名+密码注册（bcrypt 哈希），登录返回 JWT（7天有效），Bearer Token 认证
- **配置**：`config/auth.json`（`jwt_secret` 字段）
- **特点**：多用户支持，持久化，支持注册/登录/获取用户信息

```rust
pub struct AuthDb {
    conn: Mutex<Connection>,
    jwt_secret: String,
}
```

#### 数据库 Schema

`auth.db` 在插件初始化时自动创建 `users` 表：

```sql
CREATE TABLE IF NOT EXISTS users (
    id         TEXT PRIMARY KEY,       -- UUID v4，用户唯一标识
    username   TEXT NOT NULL UNIQUE,   -- 登录用户名，全局唯一
    password   TEXT NOT NULL,          -- bcrypt 哈希后的密码
    roles      TEXT NOT NULL DEFAULT 'viewer',  -- 逗号分隔的角色字符串
    created_at INTEGER NOT NULL         -- Unix 时间戳（秒），注册时间
);
```

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `id` | TEXT | PRIMARY KEY | UUID v4 字符串，由 `uuid::Uuid::new_v4()` 生成 |
| `username` | TEXT | NOT NULL UNIQUE | 注册时 trim 去除首尾空白，空字符串会被拒绝 |
| `password` | TEXT | NOT NULL | bcrypt 哈希值，使用 `DEFAULT_COST`（12）轮 |
| `roles` | TEXT | NOT NULL, DEFAULT `'viewer'` | 逗号分隔的角色名，如 `"viewer,editor"`。注册时固定为 `"viewer"` |
| `created_at` | INTEGER | NOT NULL | `chrono::Utc::now().timestamp()` 生成的 Unix 秒级时间戳 |

#### 注册流程

1. **输入校验**：检查 `username.trim()` 非空且 `password.len() >= 4`，不满足则返回错误 `"username empty or password too short (min 4)"`
2. **生成 ID**：`uuid::Uuid::new_v4().to_string()` 生成全局唯一用户 ID
3. **密码哈希**：`bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)` 对明文密码做 12 轮 bcrypt 哈希
4. **写入数据库**：执行 `INSERT INTO users (id, username, password, roles, created_at) VALUES (?1,?2,?3,?4,?5)`，roles 固定为 `"viewer"`，`created_at` 取当前 UTC 时间戳
5. **返回结果**：返回 `UserInfo { id, username, roles: vec!["viewer"] }`

```rust
pub fn register(&self, req: &RegisterRequest) -> anyhow::Result<UserInfo> {
    if req.username.trim().is_empty() || req.password.len() < 4 {
        anyhow::bail!("username empty or password too short (min 4)");
    }
    let conn = self.conn.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let hashed = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO users (id, username, password, roles, created_at) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![id, req.username.trim(), hashed, "viewer", now],
    )?;
    Ok(UserInfo { id, username: req.username.trim().to_string(), roles: vec!["viewer".into()] })
}
```

#### 登录流程

1. **查询用户**：`SELECT id, username, password, roles FROM users WHERE username = ?1`，用户名不存在则返回 `"invalid username or password"`（不区分「用户不存在」和「密码错误」，防止用户枚举攻击）
2. **密码验证**：`bcrypt::verify(&req.password, &hashed)` 对比明文与 bcrypt 哈希
3. **角色解析**：将 DB 中的逗号分隔字符串 `"viewer,editor"` 拆分为 `Vec<String>`，过滤空串
4. **构建 JWT Claims**：填充 `sub`（用户 ID）、`username`、`roles`、`exp`（当前时间 + 7天秒数）、`iat`（签发时间）
5. **签发 Token**：`jsonwebtoken::encode()` 使用 HS256 算法和 `jwt_secret` 签名
6. **返回结果**：返回 `AuthResponse { token, user: UserInfo { id, username, roles } }`

```rust
pub fn login(&self, req: &LoginRequest) -> anyhow::Result<AuthResponse> {
    let conn = self.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, username, password, roles FROM users WHERE username = ?1"
    )?;
    let (id, username, hashed, roles_str): (String, String, String, String) =
        stmt.query_row([req.username.trim()], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|_| anyhow::anyhow!("invalid username or password"))?;
    if !bcrypt::verify(&req.password, &hashed)? {
        anyhow::bail!("invalid username or password");
    }
    let roles: Vec<String> = roles_str.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: id.clone(), username: username.clone(), roles: roles.clone(),
        exp: now + 86400 * 7, iat: now,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_bytes()),
    )?;
    Ok(AuthResponse { token, user: UserInfo { id, username, roles } })
}
```

#### JWT Claims 结构

```rust
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,       // Subject — 用户 UUID（对应 users.id）
    username: String,  // 用户名
    roles: Vec<String>, // 角色列表，如 ["viewer", "editor"]
    exp: usize,        // Expiration Time — 过期时间（Unix 秒），签发时设为 now + 604800（7天）
    iat: usize,        // Issued At — 签发时间（Unix 秒）
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `sub` | String | Subject，用户唯一标识（UUID v4） |
| `username` | String | 登录用户名 |
| `roles` | `Vec<String>` | 角色列表，从 DB 的逗号分隔字符串解析而来 |
| `exp` | usize | 过期时间，Unix 秒。登录时 = `now + 86400 * 7`；`create_session` 时 = `now + 86400`（1天） |
| `iat` | usize | 签发时间，Unix 秒 |

#### Token 验证

`authenticate()` 方法实现 `AuthPlugin` trait 的请求认证入口：

1. **提取 Token**：从请求头的 `authorization` 字段中提取 `Bearer <token>`（也支持 `x-session-token` 头作为备选）
2. **空 Token 处理**：token 为空字符串时直接返回 `None`（匿名用户）
3. **解码 JWT**：调用 `check_session()` → `verify_token()`，使用 `jsonwebtoken::decode::<Claims>()` 解码，验证签名和过期时间
4. **构建 Identity**：解码成功后将 `claims.sub` 作为 `Identity.id`、`claims.roles` 直接作为 `Identity.roles`、`claims.username` 存入 `Identity.metadata`

```rust
async fn authenticate(&self, headers: &HashMap<String, String>) -> Option<Identity> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.strip_prefix("Bearer "))
        .or_else(|| headers.get("x-session-token").map(|s| s.as_str()))
        .unwrap_or("");
    if token.is_empty() { return None; }
    self.check_session(token).await
}

fn verify_token(&self, token: &str) -> Option<Identity> {
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(self.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    ).ok()?;
    let claims = data.claims;
    let mut meta = HashMap::new();
    meta.insert("username".into(), claims.username);
    Some(Identity {
        id: claims.sub,
        roles: claims.roles,
        permissions: vec![],
        metadata: meta,
    })
}
```

#### 配置

`config/auth.json` 是认证配置入口。`main.rs` 检测该文件是否存在来决定使用 `auth-db` 还是 `auth-simple`：

```json
{
  "jwt_secret": "change-this-to-a-random-secret-key-in-production"
}
```

- `jwt_secret` 为**可选字段**（`Option<String>`）。如果未提供，插件自动用 `uuid::Uuid::new_v4()` 生成一个随机密钥（进程生命周期内不变，重启后会变更——已签发的 JWT 全部失效）
- 建议生产环境手动设置一个强随机字符串，例如 `openssl rand -base64 64` 生成

#### 密码约束

- **最小长度**：`req.password.len() < 4` 时拒绝注册，返回 400 错误
- **哈希成本**：`bcrypt::DEFAULT_COST`（常量值 12），在安全性与注册/登录延迟之间取平衡
- 用户名空字符串也会被拒绝（`username.trim().is_empty()` 检查）

#### 与 roles.json 的关系

- **注册角色**：新注册用户的 `roles` 字段固定为 `"viewer"`，对应 `config/roles.json` 中定义的 `viewer` 角色（拥有 `doc:read` 权限）。管理员可通过直接修改 SQLite 数据库将用户的 `roles` 字段改为 `"viewer,editor"` 等形式来提升权限（尚无管理 UI）
- **角色存储格式**：`users.roles` 是逗号分隔的字符串（如 `"viewer,editor,admin"`），登录时通过 `split(',')` 解析为 `Vec<String>`，空字符串被过滤
- **Identity 构建**：JWT 中携带角色列表 → `verify_token()` 将其填入 `Identity.roles`；`permissions` 字段初始为空 `vec![]`，后续由 `RoleConfig::resolve_permissions()` 展开继承链并注入实际权限字符串

#### API 端点

以下端点注册在 `main.rs` 的路由表中（路径为 `handle_403` 模式，见 [鉴权模型](#鉴权模型)）：

| 方法 | 路径 | Handler | 说明 |
|------|------|---------|------|
| POST | `/api/auth/register` | `handlers::auth_handler::register` | 注册新用户，body: `{"username":"...","password":"..."}` |
| POST | `/api/auth/login` | `handlers::auth_handler::login` | 登录，返回 JWT token + 用户信息 |
| GET | `/api/auth/me` | `handlers::auth_handler::me` | 获取当前用户信息，需要 `Authorization: Bearer <token>` 头 |

`register` 成功返回 200 + `{"success":true,"data":{...}}`，失败返回 400。`login` 和 `me` 认证失败返回 401。

#### curl 示例

注册新用户：

```bash
curl -s -X POST http://127.0.0.1:3001/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
# 响应：{"success":true,"data":{"id":"...","username":"alice","roles":["viewer"]}}
```

登录获取 Token：

```bash
curl -s -X POST http://127.0.0.1:3001/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
# 响应：{"success":true,"data":{"token":"eyJhbG...","user":{...}}}
```

用 Token 获取当前用户信息：

```bash
curl -s http://127.0.0.1:3001/api/auth/me \
  -H "Authorization: Bearer eyJhbG..."
# 响应：{"success":true,"data":{"id":"...","roles":["viewer"],"username":"alice"}}
```

#### 安全提示

- **更换 jwt_secret**：生产部署必须在 `config/auth.json` 中设置一个强随机密钥（例如 `openssl rand -base64 64`），默认值和自动生成的 UUID 都不安全
- **密码强度**：bcrypt 的 12 轮哈希能抵抗暴力破解，但用户仍应使用足够复杂的密码。框架目前仅要求最低 4 字符，无复杂度策略
- **JWT 无状态**：JWT 在签发后无法服务端撤销（`destroy_session` 直接返回 `Ok(())`）。这意味着即使用户"登出"，已签发的 Token 在过期前仍然有效。如果场景要求即时吊销，需在 auth-db 之上加一层 Token 黑名单机制
- **错误信息一致**：登录失败时始终返回 `"invalid username or password"`，不区分用户不存在和密码错误，防止用户名枚举

### storage-fs（文件系统存储）

- 路径：`backend/plugins/storage-fs/`
- 依赖：`ducia-core`、`serde_json`
- 原理：`data/docs.json` 存储元数据索引，`docs/` 目录存储 `.md` 文件

**数据布局**：

```
data/
  docs.json          ← 文档索引（所有元数据）
docs/
  1.md               ← id=1 的文档内容
  2.md               ← id=2 的文档内容
```

- `data/docs.json`：存储全局索引，包含 `next_id` 自增计数器和 `docs` 映射表
- `docs/{id}.md`：每个文档的 Markdown 正文，文件名由 `DocMeta.file` 字段决定

**docs.json 结构示例**：

```json
{
  "next_id": 4,
  "docs": {
    "1": {
      "id": 1,
      "title": "Getting Started",
      "file": "1.md",
      "created_at": "2025-06-01T10:00:00Z",
      "deprecated": false,
      "deleted": false
    },
    "2": {
      "id": 2,
      "title": "API Reference",
      "file": "2.md",
      "created_at": "2025-06-02T14:30:00Z",
      "deprecated": false,
      "deleted": false
    },
    "3": {
      "id": 3,
      "title": "Changelog (已弃用)",
      "file": "3.md",
      "created_at": "2025-06-03T09:00:00Z",
      "deprecated": true,
      "deleted": false
    }
  }
}
```

**DocMeta 字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `u32` | 文档唯一标识，自增分配 |
| `title` | `String` | 文档标题 |
| `file` | `String` | 对应的 Markdown 文件名（如 `"1.md"`） |
| `created_at` | `String` | ISO 8601 创建时间 |
| `deprecated` | `bool` | 弃用标记：`true` 时前端显示弃用提示但仍可访问 |
| `deleted` | `bool` | 软删除标记：`true` 时从列表中隐藏（不返回给前端） |

**读写流程**：

1. **list_docs**：读取整个 `docs.json` → 反序列化 → 过滤 `deprecated`/`deleted`（取决于 `include_deleted` 参数）→ 返回 `Vec<DocMeta>`
2. **get_doc**：先查 `docs.json` 获取 `DocMeta` → 读取 `docs/{file}` 获取 Markdown 正文 → 组装 `DocFull { meta, content }`
3. **create_doc**：读取 `docs.json` 获取 `next_id`（如 `4`）→ 将内容写入 `docs/4.md` → 将新条目插入 `docs.json` 的 `docs` 映射 → `next_id` 自增为 `5` → 写回整个 `docs.json`
4. **update_meta**：读取 `docs.json` → 定位目标 `id` → 修改 `deprecated`/`deleted` 字段 → 写回整个 `docs.json`

**局限性**：

- **非原子操作**：读取和写入之间无锁保护，并发写入可能导致数据丢失
- **全量索引重写**：每次 `create_doc` 或 `update_meta` 都会序列化并写回整个 `docs.json`，文档数量上万时性能下降明显
- **无搜索能力**：不支持全文搜索或按标题过滤，只能遍历所有条目
- **无分页**：`list_docs` 一次返回全部文档，无法按页获取

**适用场景**：

- 单机部署、文档数量在百级以内
- 希望用 Git 对文档进行版本管理
- 需要手动编辑 Markdown 文件或 `docs.json`

### storage-sqlite（SQLite 存储）

- 路径：`backend/plugins/storage-sqlite/`
- 依赖：`ducia-core`、`rusqlite`
- 原理：文档元数据存入 SQLite，文档正文仍以文件形式存储于 `docs/`

**表结构**：

```sql
-- 文档元数据表
CREATE TABLE IF NOT EXISTS docs (
    id          INTEGER PRIMARY KEY,
    title       TEXT NOT NULL,
    file        TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    deprecated  INTEGER NOT NULL DEFAULT 0,   -- 0 = false, 1 = true
    deleted     INTEGER NOT NULL DEFAULT 0    -- 0 = false, 1 = true
);

-- 全局设置表（键值对）
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

**site_name 存储**：

`schema.sql` 执行后，插件在初始化时执行：

```sql
INSERT OR IGNORE INTO settings (key, value) VALUES ('site_name', 'My Docs');
```

`site_name()` 方法从 `settings` 表读取：

```sql
SELECT value FROM settings WHERE key = 'site_name';
```

**相比 storage-fs 的优势**：

| 特性 | storage-fs | storage-sqlite |
|------|-----------|----------------|
| 原子性 | ❌ 读写分离，无事务 | ✅ SQLite 事务保证 |
| 排序/过滤 | ❌ 只能遍历所有条目 | ✅ `ORDER BY`、`WHERE` 子句 |
| 并发安全 | ❌ 无锁保护 | ✅ `Mutex<Connection>` 串行化访问 |
| 分页 | ❌ 全量返回 | ✅ `LIMIT ... OFFSET` |
| 元数据查询 | 读整个 JSON 反序列化 | SQL 按需查询 |

**文档内容存储**：

文档正文（Markdown 内容）**不存入 SQLite**，仍以 `.md` 文件形式存储在 `docs/` 目录。这是一个刻意的设计选择：

- Markdown 内容通常较大（几 KB 到几十 KB），存入数据库会增加 I/O 开销
- 文件形式方便用外部编辑器直接修改
- 索引（元数据）在数据库中，正文在文件系统中——各取所长

**从 storage-fs 迁移到 storage-sqlite**：

1. 在 `config/settings.json` 中设置 `"use_database": true`：
   ```json
   {
     "site_name": "My Docs",
     "use_database": true
   }
   ```
2. 重启服务。插件会自动执行 `schema.sql` 建表，并从 `data/docs.json` 导入现有文档元数据
3. 验证迁移结果：通过 API 获取文档列表，确认所有文档可正常访问
4. 迁移完成后 `data/docs.json` 不再使用（可保留作为备份）

## 如何编写一个新插件

### 完整的编写清单

无论编写存储插件还是认证插件，流程完全一致：

1. **确定要实现哪个 trait**：存储插件实现 `DocRepository`，认证插件实现 `AuthPlugin`
2. **创建 crate**：`backend/plugins/<plugin-name>/`
3. **编写 `Cargo.toml`**：依赖 `ducia-core` + 所需的第三方库
4. **定义结构体**：包含连接、配置等状态字段
5. **实现 trait**：`#[async_trait] impl Xxx for Yyy { ... }`
6. **在 server 中注册**：修改 `backend/server/Cargo.toml` 和 `backend/server/src/main.rs`
7. **验证**：`cargo check`、启动服务、用 curl 测试

> **编写认证插件也一样**：把上面第 2 步中的 `DocRepository` 换成 `AuthPlugin`，其余步骤完全相同。

### 认证插件示例：auth-ldap 概念实现

以 LDAP 认证为例，展示 `AuthPlugin` trait 的实现模式：

```rust
// backend/plugins/auth-ldap/src/lib.rs
use async_trait::async_trait;
use ducia_core::plugin::auth::{AuthPlugin, Identity};
use ldap3::{LdapConn, LdapConnSettings};
use std::collections::HashMap;

pub struct LdapAuth {
    server_url: String,
    base_dn: String,
}

impl LdapAuth {
    pub fn new(server_url: &str, base_dn: &str) -> Self {
        Self {
            server_url: server_url.to_string(),
            base_dn: base_dn.to_string(),
        }
    }
}

#[async_trait]
impl AuthPlugin for LdapAuth {
    fn name(&self) -> &str {
        "auth-ldap"
    }

    async fn authenticate(&self, headers: &HashMap<String, String>) -> Option<Identity> {
        // 从 Authorization 头提取 Basic Auth 凭据
        let auth_header = headers.get("authorization")?;
        let (username, password) = parse_basic_auth(auth_header)?;

        // 连接 LDAP 服务器并尝试绑定
        let mut ldap = LdapConn::new(&self.server_url).ok()?;
        let bind_dn = format!("uid={},{}", username, self.base_dn);
        ldap.simple_bind(&bind_dn, &password).ok()?;

        Some(Identity {
            id: username,
            roles: vec!["authenticated".into()],
            permissions: vec!["doc:read".into(), "doc:write".into()],
            metadata: HashMap::new(),
        })
    }

    async fn create_session(&self, identity: &Identity) -> anyhow::Result<String> {
        // LDAP 认证一般不管理会话——直接返回身份 id 作为 token
        Ok(identity.id.clone())
    }

    async fn destroy_session(&self, _token: &str) -> anyhow::Result<()> {
        // 无状态，无需销毁
        Ok(())
    }

    async fn check_session(&self, _token: &str) -> Option<Identity> {
        // 无状态会话，不支持验证
        None
    }
}

fn parse_basic_auth(header: &str) -> Option<(String, String)> {
    let encoded = header.strip_prefix("Basic ")?;
    let decoded = String::from_utf8(
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded).ok()?
    ).ok()?;
    let (user, pass) = decoded.split_once(':')?;
    Some((user.to_string(), pass.to_string()))
}
```

### 存储插件完整示例：storage-redis

以编写一个 `storage-redis` 为例：

### 1. 创建 crate

```bash
mkdir -p backend/plugins/storage-redis/src
```

### 2. 编写 Cargo.toml

```toml
[package]
name = "ducia-storage-redis"
version = "0.1.0"
edition = "2024"

[dependencies]
ducia-core = { path = "../../ducia-core" }
redis = "0.25"
async-trait = "0.1"
anyhow = "1"
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

### 3. 实现 DocRepository trait

```rust
// src/lib.rs
use async_trait::async_trait;
use ducia_core::doc::model::{CreateDocRequest, DocFull, DocMeta};
use ducia_core::doc::repo::DocRepository;
use redis::Client;

pub struct RedisStorage {
    client: Client,
}

impl RedisStorage {
    pub fn new(redis_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            client: Client::open(redis_url)?,
        })
    }
}

#[async_trait]
impl DocRepository for RedisStorage {
    fn name(&self) -> &str {
        "storage-redis"
    }

    async fn list_docs(&self, include_deleted: bool) -> anyhow::Result<Vec<DocMeta>> {
        // 从 Redis 中读取文档列表...
        todo!()
    }

    async fn get_doc(&self, id: &str) -> anyhow::Result<Option<DocFull>> {
        // 从 Redis 中读取单个文档...
        todo!()
    }

    async fn create_doc(&self, req: CreateDocRequest) -> anyhow::Result<DocMeta> {
        // 将文档存入 Redis...
        todo!()
    }

    async fn update_meta(
        &self, id: &str,
        deprecated: Option<bool>,
        deleted: Option<bool>,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn site_name(&self) -> String {
        "My Redis Site".into()
    }
}
```

### 4. 在 server 中注册

在 `backend/server/Cargo.toml` 添加依赖：

```toml
ducia-storage-redis = { path = "../plugins/storage-redis" }
```

在 `backend/server/src/main.rs` 中注册：

```rust
use ducia_storage_redis::RedisStorage;

let storage_plugin: StoragePlugin = Box::new(
    RedisStorage::new("redis://127.0.0.1:6379")?
);
```

### 5. 验证

完成以上步骤后，按顺序验证：

```bash
# 1. 编译检查（确保类型和 trait 实现正确）
cargo check -p ducia-storage-redis

# 2. 检查 server 能否正确链接新插件
cargo check -p ducia-server

# 3. 启动服务
cargo run -p ducia-server

# 4. 用 curl 测试文档列表
curl -s http://127.0.0.1:3001/api/docs

# 5. 测试创建文档
curl -s -X POST http://127.0.0.1:3001/api/docs \
  -H "Content-Type: application/json" \
  -d '{"title":"Test Doc","content":"# Hello Redis"}'
```

| 验证步骤 | 命令 | 预期结果 |
|----------|------|----------|
| `cargo check` | `cargo check -p <plugin-name>` | 零错误、零警告 |
| 添加到 Cargo.toml | 编辑 `backend/server/Cargo.toml` | 在 `[dependencies]` 中正确引用 |
| 注册到 main.rs | 编辑 `backend/server/src/main.rs` | `use` 导入 + `PluginRegistry` 注册 |
| 启动服务 | `cargo run -p ducia-server` | 服务正常启动，无 panic |
| curl 测试 | `curl http://127.0.0.1:3001/api/docs` | 返回 JSON，包含文档列表 |

同样的流程适用于编写新的认证插件——只需实现 `AuthPlugin` trait 并在 `main.rs` 中注册即可。
