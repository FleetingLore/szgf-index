//! auth-simple — 序列码认证插件
//!
//! 这是最简单的认证方式：
//! - 通过 config/sequence.json 定义序列
//! - 前端按序点击按钮输入序列
//! - 成功后服务端发放 UUID token
//! - token 存储在内存 HashMap 中

use anyhow::Result;
use async_trait::async_trait;
use ducia_core::perm::model::Identity;
use ducia_core::plugin::auth::AuthPlugin;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Mutex;

/// 序列码认证插件
pub struct SimpleAuth {
    config_dir: PathBuf,
    sessions: Mutex<HashMap<String, Identity>>,
}

impl SimpleAuth {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// 从 config/sequence.json 加载验证序列
    pub fn load_sequence(&self) -> Vec<usize> {
        let path = self.config_dir.join("sequence.json");
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(arr) = json.get("sequence").and_then(|s| s.as_array()) {
                    return arr
                        .iter()
                        .filter_map(|v| v.as_u64())
                        .map(|v| v as usize)
                        .collect();
                }
            }
        }
        Vec::new()
    }
}

#[async_trait]
impl AuthPlugin for SimpleAuth {
    fn name(&self) -> &str {
        "auth-simple"
    }

    async fn authenticate(&self, headers: &HashMap<String, String>) -> Option<Identity> {
        let token = headers
            .get("x-session-token")
            .map(|s| s.as_str())
            .unwrap_or("");

        if token.is_empty() {
            return None;
        }

        self.check_session(token).await
    }

    async fn create_session(&self, identity: &Identity) -> Result<String> {
        let token = uuid::Uuid::new_v4().to_string();
        self.sessions
            .lock()
            .await
            .insert(token.clone(), identity.clone());
        Ok(token)
    }

    async fn destroy_session(&self, token: &str) -> Result<()> {
        self.sessions.lock().await.remove(token);
        Ok(())
    }

    async fn check_session(&self, token: &str) -> Option<Identity> {
        self.sessions.lock().await.get(token).cloned()
    }
}
