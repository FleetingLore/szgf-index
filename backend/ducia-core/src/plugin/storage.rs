//! 存储插件 trait
//!
//! 文档持久化的抽象。默认提供 filesystem 实现，
//! 可替换为 SQLite、PostgreSQL、S3 等。

use crate::doc::repo::DocRepository;

/// 存储插件别名——它就是一个实现了 DocRepository 的类型
///
/// 使用 trait object 包装以在插件注册表中统一管理。
pub type StoragePlugin = Box<dyn DocRepository>;
