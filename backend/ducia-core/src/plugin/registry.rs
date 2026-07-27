//! 插件注册表
//!
//! 管理所有已注册的插件，提供按类型查找的能力。

use crate::plugin::auth::AuthPlugin;
use crate::plugin::storage::StoragePlugin;
use std::collections::HashMap;
use std::sync::Arc;

/// 插件注册表
///
/// 在应用启动时组装，整个生命周期内不变。
pub struct PluginRegistry {
    /// 认证插件（框架约定只有一个活跃的认证插件）
    auth: Option<Arc<dyn AuthPlugin>>,
    /// 存储插件
    storage: Option<StoragePlugin>,
    /// 额外插件（按名称索引）
    extras: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl PluginRegistry {
    /// 创建空的注册表
    pub fn new() -> Self {
        Self {
            auth: None,
            storage: None,
            extras: HashMap::new(),
        }
    }

    /// 注册认证插件
    pub fn with_auth(mut self, plugin: Arc<dyn AuthPlugin>) -> Self {
        self.auth = Some(plugin);
        self
    }

    /// 注册存储插件
    pub fn with_storage(mut self, plugin: StoragePlugin) -> Self {
        self.storage = Some(plugin);
        self
    }

    /// 获取认证插件
    pub fn auth(&self) -> Option<&Arc<dyn AuthPlugin>> {
        self.auth.as_ref()
    }

    /// 获取存储插件
    pub fn storage(&self) -> Option<&StoragePlugin> {
        self.storage.as_ref()
    }

    /// 注册任意额外插件（供未来扩展）
    pub fn register_extra<T: 'static + Send + Sync>(&mut self, name: &str, plugin: T) {
        self.extras.insert(name.to_string(), Box::new(plugin));
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
