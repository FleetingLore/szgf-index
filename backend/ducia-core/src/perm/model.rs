//! 权限模型
//!
//! 框架不预设 admin/editor/viewer 等角色。
//! 角色由部署者通过 `config/roles.json` 动态定义，
//! 框架只理解 `resource:action` 格式的权限字符串。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 权限字符串，格式 `"resource:action"`
///
/// 例如：`"doc:read"` `"doc:write"` `"user:manage"` `"*"`（通配）
pub type Permission = String;

/// 用户身份
///
/// 由认证插件在每次请求时构建，不持久化。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// 用户唯一标识
    pub id: String,
    /// 拥有的角色列表（如 `["admin", "editor"]`）
    pub roles: Vec<String>,
    /// 直接授予的权限（如 `["doc:read"]`）
    pub permissions: Vec<Permission>,
    /// 自由扩展的元数据
    pub metadata: HashMap<String, String>,
}

impl Identity {
    /// 检查是否拥有某个权限
    ///
    /// 检查逻辑：
    /// 1. 如果用户有 `"*"` 权限，直接通过
    /// 2. 如果用户有匹配的直接权限，通过
    pub fn has_permission(&self, required: &str) -> bool {
        // 通配符
        if self.permissions.iter().any(|p| p == "*") {
            return true;
        }
        // 精确匹配
        self.permissions.iter().any(|p| p == required)
    }

    /// 是否拥有某个角色
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

/// 角色定义（从 roles.json 加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDef {
    /// 角色拥有的权限
    #[serde(default)]
    pub permissions: Vec<Permission>,
    /// 继承的角色（权限合并）
    #[serde(default)]
    pub extends: Vec<String>,
}

/// 角色配置（roles.json 的顶层结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConfig {
    pub roles: HashMap<String, RoleDef>,
}

impl RoleConfig {
    /// 解析一个身份的实际权限（展开继承链）
    ///
    /// 遍历用户的所有角色，递归收集其权限。
    pub fn resolve_permissions(&self, identity: &Identity) -> HashSet<Permission> {
        let mut perms: HashSet<Permission> = identity.permissions.iter().cloned().collect();
        let mut seen_roles: HashSet<String> = HashSet::new();

        for role in &identity.roles {
            self.collect_permissions(role, &mut perms, &mut seen_roles);
        }

        perms
    }

    fn collect_permissions(
        &self,
        role: &str,
        perms: &mut HashSet<Permission>,
        seen: &mut HashSet<String>,
    ) {
        if !seen.insert(role.to_string()) {
            return; // 防止循环继承
        }

        if let Some(def) = self.roles.get(role) {
            perms.extend(def.permissions.iter().cloned());

            for parent in &def.extends {
                self.collect_permissions(parent, perms, seen);
            }
        }
    }

    /// 构建一个完整的 Identity（合并角色权限）
    pub fn build_identity(&self, id: &str, roles: Vec<String>) -> Identity {
        let identity = Identity {
            id: id.to_string(),
            roles: roles.clone(),
            permissions: vec![],
            metadata: HashMap::new(),
        };

        let resolved = self.resolve_permissions(&identity);

        Identity {
            id: id.to_string(),
            roles,
            permissions: resolved.into_iter().collect(),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_permissions() {
        let id = Identity {
            id: "test".into(),
            roles: vec![],
            permissions: vec!["doc:read".into(), "doc:write".into()],
            metadata: HashMap::new(),
        };

        assert!(id.has_permission("doc:read"));
        assert!(id.has_permission("doc:write"));
        assert!(!id.has_permission("doc:delete"));
    }

    #[test]
    fn test_wildcard_permission() {
        let id = Identity {
            id: "admin".into(),
            roles: vec!["admin".into()],
            permissions: vec!["*".into()],
            metadata: HashMap::new(),
        };

        assert!(id.has_permission("anything"));
    }

    #[test]
    fn test_role_inheritance() {
        let mut roles = HashMap::new();
        roles.insert(
            "editor".into(),
            RoleDef {
                permissions: vec!["doc:write".into()],
                extends: vec!["viewer".into()],
            },
        );
        roles.insert(
            "viewer".into(),
            RoleDef {
                permissions: vec!["doc:read".into()],
                extends: vec![],
            },
        );

        let config = RoleConfig { roles };

        let identity = config.build_identity("test", vec!["editor".into()]);
        assert!(identity.has_permission("doc:read"));
        assert!(identity.has_permission("doc:write"));
        assert!(!identity.has_permission("doc:delete"));
    }
}
