//! auth-db — 数据库支持的内置身份系统
//!
//! 功能：
//! - 用户注册（用户名 + 密码，bcrypt 哈希）
//! - 用户登录（返回 JWT token）
//! - 会话验证（AuthPlugin trait）
//! - 简单角色分配（默认 "viewer"，管理员可提升）
//!
//! # API
//!
//! POST /api/auth/register  {"username":"...","password":"..."}
//! POST /api/auth/login     {"username":"...","password":"..."}
//! GET  /api/auth/me        Authorization: Bearer <token>
//!
//! # 配置 config/auth.json
//!
//! ```json
//! { "jwt_secret": "your-secret-key-here-keep-it-safe" }
//! ```
//!
//! # 外部认证
//!
//! 如需接入外部身份系统，使用 auth-external 插件，
//! 配置外部验证 URL。auth-db 专注本地认证。

use async_trait::async_trait;
use ducia_core::perm::model::Identity;
use ducia_core::plugin::auth::AuthPlugin;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

// ═══════ 数据模型 ═══════

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    username: String,
    roles: Vec<String>,
    exp: usize,
    iat: usize,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: Option<String>,
}

// ═══════ AuthDb 插件 ═══════

pub struct AuthDb {
    conn: Mutex<Connection>,
    jwt_secret: String,
}

impl AuthDb {
    pub fn new(db_path: PathBuf, config: AuthConfig) -> anyhow::Result<Self> {
        let _ = std::fs::create_dir_all(db_path.parent().unwrap_or(&db_path));
        let conn = Connection::open(&db_path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                id         TEXT PRIMARY KEY,
                username   TEXT NOT NULL UNIQUE,
                password   TEXT NOT NULL,
                roles      TEXT NOT NULL DEFAULT 'viewer',
                created_at INTEGER NOT NULL
            );",
        )?;

        let jwt_secret = config
            .jwt_secret
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        Ok(Self {
            conn: Mutex::new(conn),
            jwt_secret,
        })
    }

    // ── 注册 ──

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

        Ok(UserInfo {
            id,
            username: req.username.trim().to_string(),
            roles: vec!["viewer".into()],
        })
    }

    // ── 登录 ──

    pub fn login(&self, req: &LoginRequest) -> anyhow::Result<AuthResponse> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, username, password, roles FROM users WHERE username = ?1")?;

        let (id, username, hashed, roles_str): (String, String, String, String) = stmt
            .query_row([req.username.trim()], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|_| anyhow::anyhow!("invalid username or password"))?;

        if !bcrypt::verify(&req.password, &hashed)? {
            anyhow::bail!("invalid username or password");
        }

        let roles: Vec<String> = roles_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: id.clone(),
            username: username.clone(),
            roles: roles.clone(),
            exp: now + 86400 * 7,
            iat: now,
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )?;

        Ok(AuthResponse {
            token,
            user: UserInfo {
                id,
                username,
                roles,
            },
        })
    }

    // ── 验证 ──

    fn verify_token(&self, token: &str) -> Option<Identity> {
        let data = jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .ok()?;

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
}

// ═══════ AuthPlugin trait ═══════

#[async_trait]
impl AuthPlugin for AuthDb {
    fn name(&self) -> &str {
        "auth-db"
    }

    async fn authenticate(&self, headers: &HashMap<String, String>) -> Option<Identity> {
        let token = headers
            .get("authorization")
            .and_then(|v| v.strip_prefix("Bearer "))
            .or_else(|| headers.get("x-session-token").map(|s| s.as_str()))
            .unwrap_or("");
        if token.is_empty() {
            return None;
        }
        self.check_session(token).await
    }

    async fn create_session(&self, identity: &Identity) -> anyhow::Result<String> {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: identity.id.clone(),
            username: identity
                .metadata
                .get("username")
                .cloned()
                .unwrap_or_default(),
            roles: identity.roles.clone(),
            exp: now + 86400,
            iat: now,
        };
        Ok(jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )?)
    }

    async fn destroy_session(&self, _token: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn check_session(&self, token: &str) -> Option<Identity> {
        self.verify_token(token)
    }
}
