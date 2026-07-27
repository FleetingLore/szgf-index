//! ducia-core — 框架核心类型、trait 和插件系统
//!
//! 这个 crate 不包含任何具体实现，只定义接口契约。
//! 所有业务逻辑通过插件注入，框架本体保持极简。
//!
//! # 架构
//!
//! ```text
//! ducia-core
//! ├── doc     — 文档模型 + 存储 trait
//! ├── plugin  — 插件注册表 + Auth/Storage/Visibility trait
//! ├── perm    — 动态权限引擎
//! └── i18n    — 国际化支持
//! ```

pub mod doc;
pub mod i18n;
pub mod perm;
pub mod plugin;

// 从各模块重新导出常用类型
pub use doc::model::*;
pub use doc::repo::DocRepository;
pub use i18n::I18nManager;
pub use perm::model::*;
pub use plugin::auth::AuthPlugin;
pub use plugin::registry::PluginRegistry;
pub use plugin::storage::StoragePlugin;
