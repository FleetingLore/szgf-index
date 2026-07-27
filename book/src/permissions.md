# 权限模型

Ducia 采用**动态角色 + 继承链 + 通配符**的权限模型。框架不预设 `admin`、`editor`、`viewer` 等角色，所有角色和权限由部署者在 `config/roles.json` 中自由定义。

## 设计原则

1. **框架不预设角色**：admin/viewer/editor 只是配置中的名字，框架不识别它们
2. **权限即字符串**：格式为 `"resource:action"`，框架只做字符串匹配
3. **角色可继承**：一个角色通过 `extends` 字段继承其他角色的权限
4. **通配符**：`"*"` 权限表示拥有所有权限
5. **循环检测**：`resolve_permissions()` 自动防止循环继承

## 核心类型

### Identity（用户身份）

每次请求由认证插件构造，包含：

```rust
pub struct Identity {
    pub id: String,                           // 用户唯一标识
    pub roles: Vec<String>,                   // 角色列表
    pub permissions: Vec<Permission>,         // 直接权限
    pub metadata: HashMap<String, String>,    // 扩展元数据
}
```

### RoleDef（角色定义）

```rust
pub struct RoleDef {
    pub permissions: Vec<Permission>,  // 角色自带的权限
    pub extends: Vec<String>,          // 继承的角色名
}
```

### RoleConfig（角色配置）

```rust
pub struct RoleConfig {
    pub roles: HashMap<String, RoleDef>,  // 角色名 → 角色定义
}
```

## 权限格式

权限字符串采用 `"resource:action"` 格式：

| 权限字符串 | 含义 |
|-----------|------|
| `"doc:read"` | 阅读文档 |
| `"doc:write"` | 创建/编辑文档 |
| `"doc:delete"` | 删除文档 |
| `"doc:deprecate"` | 标记弃用文档 |
| `"user:manage"` | 管理用户 |
| `"config:write"` | 修改配置 |
| `"*"` | 通配符，拥有所有权限 |

## 角色继承

`config/roles.json` 示例——经典的五级继承链：

```json
{
  "roles": {
    "superadmin": {
      "permissions": ["*"],
      "extends": ["admin"]
    },
    "admin": {
      "permissions": ["doc:delete", "user:manage", "config:write"],
      "extends": ["editor"]
    },
    "editor": {
      "permissions": ["doc:write", "doc:deprecate"],
      "extends": ["viewer"]
    },
    "viewer": {
      "permissions": ["doc:read"],
      "extends": []
    },
    "anonymous": {
      "permissions": ["doc:read"],
      "extends": []
    }
  }
}
```

继承关系可视化：

```text
superadmin [permissions: *]
    │  extends
    ▼
admin [+ doc:delete, user:manage, config:write]
    │  extends
    ▼
editor [+ doc:write, doc:deprecate]
    │  extends
    ▼
viewer [doc:read]
    │  extends
    ▼
anonymous [doc:read]
```

一个 `editor` 角色的用户最终拥有：`doc:write` + `doc:deprecate` + `doc:read`（继承自 viewer）。

## resolve_permissions() 递归解析

`RoleConfig::resolve_permissions()` 遍历用户的所有角色，递归收集权限：

```rust
impl RoleConfig {
    pub fn resolve_permissions(&self, identity: &Identity) -> HashSet<Permission> {
        let mut perms: HashSet<Permission> = identity.permissions.iter().cloned().collect();
        let mut seen_roles: HashSet<String> = HashSet::new();

        for role in &identity.roles {
            self.collect_permissions(role, &mut perms, &mut seen_roles);
        }
        perms
    }

    fn collect_permissions(
        &self, role: &str,
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
}
```

解析过程：

1. 从 `Identity` 的 `permissions` 字段收集直接权限
2. 遍历 `Identity` 的 `roles` 列表
3. 对每个角色，查找 `roles.json` 中的 `RoleDef`
4. 收集该角色的 `permissions`
5. 递归处理该角色的 `extends` 父角色
6. 用 `seen` 集合防止循环继承

## has_permission() 和 has_role()

`Identity` 提供两个便捷方法：

```rust
impl Identity {
    /// 检查是否拥有某个权限
    pub fn has_permission(&self, required: &str) -> bool {
        // 1. 通配符 "*" 直接通过
        if self.permissions.iter().any(|p| p == "*") {
            return true;
        }
        // 2. 精确匹配
        self.permissions.iter().any(|p| p == required)
    }

    /// 检查是否拥有某个角色
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}
```

检查逻辑分两步：

1. **通配符优先**：如果用户有 `"*"` 权限（如 superadmin），直接通过所有检查
2. **精确匹配**：在已解析的权限列表中查找匹配项

## build_identity() 构建完整身份

由于用户身份通常只有角色信息（无直接权限），框架提供 `build_identity()` 将角色展开为权限：

```rust
let identity = role_config.build_identity("user-1", vec!["editor".into()]);
// identity.permissions 现在是 ["doc:write", "doc:deprecate", "doc:read"]
// identity.has_permission("doc:write") → true
// identity.has_permission("doc:delete") → false
```

## 使用示例

### 在 handler 中进行权限检查

```rust
pub async fn upload_cat(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreateDocRequest>,
) -> impl Responder {
    // 1. 从请求中提取身份（如果未认证则为匿名）
    let identity = extract_identity(&req, &state.plugins).await
        .unwrap_or_else(ducia_core::plugin::auth::anonymous);

    // 2. 解析完整权限（展开角色继承链）
    let full_identity = state.role_config.build_identity(
        &identity.id, identity.roles
    );

    // 3. 检查权限
    if !full_identity.has_permission("doc:write") {
        return HttpResponse::Forbidden().json(json!({
            "success": false,
            "message": "permission denied: doc:write required"
        }));
    }

    // 4. 执行操作
    // ...
}
```

### 自定义角色体系

你可以完全重新定义角色体系，不限于 admin/editor/viewer：

```json
{
  "roles": {
    "teacher": {
      "permissions": ["doc:write", "doc:delete", "doc:deprecate", "user:manage"],
      "extends": ["student"]
    },
    "student": {
      "permissions": ["doc:read", "doc:write"],
      "extends": []
    },
    "guest": {
      "permissions": ["doc:read"],
      "extends": []
    }
  }
}
```

框架不关心角色叫什么名字——它只是按继承链展开权限字符串。
