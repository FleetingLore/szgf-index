//! 应用状态
//!
//! 持有插件注册表和 I18n 管理器，通过 actix-web 的 `Data` 注入。

use ducia_core::I18nManager;
use ducia_core::perm::model::RoleConfig;
use ducia_core::plugin::registry::PluginRegistry;
use std::sync::Arc;

/// 全局应用状态
pub struct AppState {
    /// 插件注册表
    pub plugins: PluginRegistry,
    /// 角色配置
    pub role_config: Arc<RoleConfig>,
    /// I18n 管理器
    pub i18n: I18nManager,
}

impl AppState {
    pub fn new(plugins: PluginRegistry, role_config: RoleConfig, i18n: I18nManager) -> Self {
        Self {
            plugins,
            role_config: Arc::new(role_config),
            i18n,
        }
    }
}
