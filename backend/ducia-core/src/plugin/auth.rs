//! 认证插件 trait
//!
//! 认证不是框架的一部分，而是可插拔的扩展点。
//! 你可以实现序列码、JWT、OAuth、LDAP 等任意认证方式。

use crate::perm::model::Identity;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// 认证插件接口
///
/// 实现此 trait 的类型可以注册为认证后端。
/// 框架不关心具体如何认证——它只调用这些方法。
///
/// 注意：trait 方法接收 owned 数据（headers, token），
/// 因为 actix-web 的 HttpRequest 内部使用 Rc，不可跨 await 边界传递。
#[async_trait]
pub trait AuthPlugin: Send + Sync {
    /// 插件唯一名称
    fn name(&self) -> &str;

    /// 从请求头中提取身份
    ///
    /// `headers` 是 caller 从 HttpRequest 中提取的 owned map。
    /// 返回 `None` 表示未认证（匿名用户）。
    async fn authenticate(&self, headers: &HashMap<String, String>) -> Option<Identity>;

    /// 创建会话
    ///
    /// 返回一个 token/凭证字符串给客户端。
    async fn create_session(&self, identity: &Identity) -> Result<String>;

    /// 销毁会话
    async fn destroy_session(&self, token: &str) -> Result<()>;

    /// 检查会话是否有效，返回对应身份
    async fn check_session(&self, token: &str) -> Option<Identity>;
}

/// 匿名身份——用于未登录用户
pub fn anonymous() -> Identity {
    Identity {
        id: "anonymous".into(),
        roles: vec!["anonymous".into()],
        permissions: vec!["doc:read".into()],
        metadata: Default::default(),
    }
}
