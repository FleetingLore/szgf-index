# 认证后端

Ducia 的认证层通过 `AuthPlugin` trait 抽象，支持即插即用的认证方案。框架默认提供两种实现，也可对接外部认证系统。

## 架构概览

```text
HTTP Request
    │
    ├── config/auth.json 存在? ──→ auth-db ──→ JWT + bcrypt ──→ data/auth.db
    │
    └── config/auth.json 不存在 ──→ auth-simple ──→ UUID token ──→ 内存 HashMap
    
    两者都实现 AuthPlugin trait → 注入 PluginRegistry → Handler 调用
```

认证插件通过 `PluginRegistry` 注入，handler 在需要认证的接口中通过内置 session endpoint 或自定义逻辑调用 `AuthPlugin` 方法。

---

## `AuthPlugin` trait

定义在 `ducia-core/plugin/auth.rs`，所有认证后端必须实现此接口：

```rust
#[async_trait]
pub trait AuthPlugin: Send + Sync {
    /// 插件唯一名称
    fn name(&self) -> &str;

    /// 从请求头中提取身份
    /// 返回 None 表示未认证（匿名用户）
    async fn authenticate(&self, headers: &HashMap<String, String>) -> Option<Identity>;

    /// 创建会话，返回 token/凭证字符串
    async fn create_session(&self, identity: &Identity) -> Result<String>;

    /// 销毁会话
    async fn destroy_session(&self, token: &str) -> Result<()>;

    /// 检查会话是否有效，返回对应身份
    async fn check_session(&self, token: &str) -> Option<Identity>;
}
```

### 核心数据类型

| 类型 | 说明 | 关键字段 |
|------|------|----------|
| `Identity` | 用户身份 | `id`, `username`(metadata), `roles`, `permissions` |
| `RoleConfig` | 角色配置 | 从 `config/roles.json` 加载的角色定义映射 |

### 身份解析流程

```
HTTP Request
  → 提取请求头 (HashMap<String, String>)
  → AuthPlugin::authenticate(headers)
  → 返回 Some(Identity) 或 None (匿名)
```

框架本身不执行访问控制检查——handler 层根据 `Identity` 中的权限自行判断。权限的解析（继承链展开）由 `RoleConfig::build_identity()` 完成。

---

## auth-simple — 序列码认证

**默认认证方案**（当 `config/auth.json` 不存在时自动启用）。

### 工作原理

```
1. 前端 GET /api/admin/sequence  →  获取 [1,2,3,2,3,4]
2. 用户在管理面板按序点击数字按钮
3. 前端验证序列正确 → POST /api/admin/session
4. 服务端生成 UUID token → 存入内存 HashMap
5. 后续请求携带 X-Session-Token 头
```

### 数制设计

管理面板是一个 2×2 的按钮网格，数字排列为 `[2, 1, 3, 4]`：

```
┌───┬───┐
│ 2 │ 1 │
├───┼───┤
│ 3 │ 4 │
└───┴───┘
```

这四个数字 **1-4** 是五进制（base-5）的非零位。五进制使用 0,1,2,3,4 五个数字，此处 0 不在网格中意味着"无输入"。

`sequence.json` 中的序列 `[1,2,3,2,3,4]` 实质上是一个五进制数字串。示例密码 `"1234"` 同样是一个五进制数——它是 1×5³+2×5²+3×5+4 = 194₁₀。两条认证路径共享同一套数制基础，只是输入方式不同：前端的按钮点击和后端的密码文本，最终都归约到同一个五进制数字空间。

### 数据结构

```rust
pub struct SimpleAuth {
    config_dir: PathBuf,
    sessions: Mutex<HashMap<String, Identity>>,
}
```

- **序列定义**：从 `config/sequence.json` 加载，内容明文可见
- **会话存储**：`tokio::sync::Mutex<HashMap<String, Identity>>` 内存存储
- **Token 格式**：UUID v4（如 `a1b2c3d4-e5f6-7890-abcd-ef1234567890`）
- **Token 生命周期**：服务重启即失效

### 认证头格式

```
X-Session-Token: a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

请求头名称采用小写匹配（`x-session-token`），HTTP 协议本身不区分大小写。

### 优点

- **零依赖**：无需数据库、无需密码
- **极简部署**：只需一个 JSON 配置文件
- **即时可用**：无注册流程

### 缺点

- **无密码保护**：序列码明文存储在配置文件中
- **仅单一管理员**：所有通过序列认证的用户都是 `admin` 角色
- **服务重启丢失**：所有会话在重启后失效
- **不适合多用户**：无法区分不同用户身份

### 适用场景

- 个人或小团队内部使用
- 受信任的内网环境
- 开发测试阶段
- 不需要多用户系统的场景

---

## auth-db — JWT + 数据库认证

**生产环境推荐方案**（创建 `config/auth.json` 即可启用）。

### 工作原理

```text
注册流程：
  客户端 → POST /api/auth/register {username, password}
  服务端 → 校验 → bcrypt 哈希 → INSERT INTO users → 返回 {id, username, roles:["viewer"]}

登录流程：
  客户端 → POST /api/auth/login {username, password}
  服务端 → SELECT user → bcrypt::verify → 签发 JWT → 返回 {token, user}

认证请求：
  客户端 → Authorization: Bearer <token>
  服务端 → JWT 解码 → 过期校验 → 业务数据
```

### 数据结构

```rust
pub struct AuthDb {
    conn: Mutex<Connection>,   // SQLite 数据库连接
    jwt_secret: String,        // JWT 签名密钥
}
```

### 数据库 Schema（`data/auth.db`）

**`users` 表**

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `id` | `TEXT` | `PRIMARY KEY` | 用户 UUID v4 |
| `username` | `TEXT` | `NOT NULL UNIQUE` | 用户名 |
| `password` | `TEXT` | `NOT NULL` | bcrypt 哈希后的密码 |
| `roles` | `TEXT` | `DEFAULT 'viewer'` | 逗号分隔的角色列表 |
| `created_at` | `INTEGER` | `NOT NULL` | 注册时间戳（秒） |

### JWT Token 结构

JWT payload（Claims）：

```json
{
  "sub": "用户 UUID",
  "username": "alice",
  "roles": ["viewer"],
  "exp": 1700000000,
  "iat": 1699395200
}
```

| 字段 | 说明 |
|------|------|
| `sub` | 用户唯一标识（subject） |
| `username` | 用户名 |
| `roles` | 角色列表 |
| `exp` | 过期时间（7 天后） |
| `iat` | 签发时间 |

### 密码安全

- **bcrypt 哈希**：使用 `bcrypt::DEFAULT_COST`（cost=12）进行哈希
- **密码强度**：至少 4 个字符
- **验证方式**：`bcrypt::verify()` 常量时间比较，防时序攻击

> **关于示例中的 `"1234"`**：这不是随意选的值。数字 1-4 是管理面板按钮网格的五进制位（参见 [auth-simple 数制设计](#数制设计)），密码 `"1234"` 与管理序列 `[1,2,3,2,3,4]` 处于同一数字空间——前者是文本密码，后者是按钮序列，但底层都是五进制数字串。`"1234"` 作为示例值既满足 4 位最低长度要求，又保持了与项目数字体系的一致性。生产环境当然要用强密码——bcrypt 负责安全存储，不依赖数字本身的强度。此外 `config/auth.json` 的 `jwt_secret` 也需从默认值更换。

### JWT 安全

- **签名算法**：HS256（HMAC-SHA256）
- **密钥来源**：`config/auth.json` → `jwt_secret` 字段，若未提供则自动生成随机 UUID
- **有效期**：
  - 用户登录：7 天（`now + 86400 * 7`）
  - 管理员会话：24 小时（`now + 86400`）
- **Token 销毁**：目前为无状态 JWT（不维护黑名单），通过配置短有效期降低风险

### 认证头格式

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

同时也兼容 `X-Session-Token` 头（向后兼容 `auth-simple` 的客户端）。

### 适用场景

- 多用户系统
- 生产环境
- 需要用户注册/登录的公开服务
- 需要角色区分的场景

---

## 认证方案对比

| 特性 | auth-simple | auth-db |
|------|-------------|---------|
| 认证方式 | 序列码 | 用户名 + 密码 |
| Token 类型 | UUID v4 | JWT (HS256) |
| Token 有效期 | 直到服务重启 | 7 天（登录）/ 24 小时（管理员） |
| 存储 | 内存 HashMap | SQLite 数据库 |
| 多用户 | ❌ (单一 admin) | ✅ |
| 角色支持 | ❌ (固定 admin) | ✅ (从 roles.json 加载) |
| 密码安全 | N/A | bcrypt 哈希 |
| 重启影响 | 所有会话丢失 | Token 仍有效 |
| 配置 | 仅需 sequence.json | 需 auth.json |
| 依赖 | 无 | rusqlite, bcrypt, jsonwebtoken |

---

## 切换认证后端

### 启用 auth-db

创建 `config/auth.json` 文件：

```json
{
  "jwt_secret": "生成一个高强度的随机字符串"
}
```

重启服务，框架自动检测到该文件并切换到 `auth-db`。

```rust
// main.rs 中的切换逻辑
let auth_plugin = if config_dir.join("auth.json").exists() {
    println!("Using auth-db (database authentication)");
    Arc::new(AuthDb::new(data_dir.join("auth.db"), auth_config)?)
} else {
    println!("Using auth-simple (sequence code authentication)");
    Arc::new(SimpleAuth::new(config_dir.clone()))
};
```

### 回退到 auth-simple

删除 `config/auth.json` 文件并重启服务。注意：

- `auth.db` 中的用户数据不会被删除，但不再可用
- 所有 `auth-db` 的 JWT token 将失效
- 恢复为序列码登录方式

### 生成安全密钥

```bash
# Linux / macOS
openssl rand -hex 32

# 或使用 uuidgen
uuidgen
```

---

## 外部认证扩展（概念）

框架的 `AuthPlugin` trait 设计为可对接任意外部认证系统。以下是 `auth-external` 插件的概念设计：

```rust
pub struct ExternalAuth {
    verify_url: String,      // 外部验证端点
    http_client: Client,     // HTTP 客户端
}

#[async_trait]
impl AuthPlugin for ExternalAuth {
    async fn authenticate(&self, headers: &HashMap<String, String>) -> Option<Identity> {
        let token = extract_token(headers)?;

        // 调用外部服务验证 token
        let resp = self.http_client
            .get(&self.verify_url)
            .bearer_auth(token)
            .send().await.ok()?;

        // 解析外部服务返回的用户信息
        resp.json::<Identity>().await.ok()
    }

    async fn check_session(&self, token: &str) -> Option<Identity> {
        // 同样调用外部验证接口
        self.verify_external(token).await
    }
    // ...
}
```

### 可对接的外部系统

| 系统 | 说明 |
|------|------|
| **OAuth 2.0 / OIDC** | Google、GitHub、微软等第三方登录 |
| **LDAP / Active Directory** | 企业目录服务 |
| **自建认证服务** | 统一的内部用户中心 |
| **WebAuthn / Passkey** | 无密码认证 |
| **SAML** | 企业单点登录（SSO） |

---

## 认证流程完整示例

### 场景一：auth-simple（序列码认证）

```bash
# 1. 获取登录序列
curl http://127.0.0.1:3001/api/admin/sequence
# → {"sequence": [1,2,3,2,3,4]}

# 2. 创建管理员会话（前端按序点击按钮后调用）
curl -X POST http://127.0.0.1:3001/api/admin/session
# → {"token": "f47ac10b-..."}

# 3. 后续请求携带 token
curl http://127.0.0.1:3001/api/admin/session \
  -H 'X-Session-Token: f47ac10b-...'
# → {"isAdmin": true}

# 4. 上传文档
curl -X POST http://127.0.0.1:3001/api/cats \
  -H 'Content-Type: application/json' \
  -H 'X-Session-Token: f47ac10b-...' \
  -d '{"title":"文档","content":"# 内容"}'
```

### 场景二：auth-db（JWT 认证）

```bash
# 1. 注册用户
curl -X POST http://127.0.0.1:3001/api/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"secure1234"}'
# → {"success":true,"data":{"id":"...","username":"alice","roles":["viewer"]}}

# 2. 登录获取 JWT
curl -X POST http://127.0.0.1:3001/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"secure1234"}'
# → {"success":true,"data":{"token":"eyJ...","user":{...}}}

# 3. 验证 token
curl http://127.0.0.1:3001/api/auth/me \
  -H 'Authorization: Bearer eyJ...'
# → {"success":true,"data":{"id":"...","username":"alice","roles":["viewer"]}}

# 4. 使用 token 访问资源
curl http://127.0.0.1:3001/api/cats \
  -H 'Authorization: Bearer eyJ...'
```
