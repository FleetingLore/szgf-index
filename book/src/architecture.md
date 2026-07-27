# 系统架构

Ducia 采用**前后端分离 + 插件化**架构，后端负责 API、认证和存储，前端负责 UI 渲染和交互。

## 整体分层

```text
浏览器 (React 18 + TS + Vite)
    │  /api/*
    ▼
Vite Dev Proxy (:5173) → Actix-web (:3001)
    │
    ▼
PluginRegistry
    ├── Auth Plugin (认证)
    ├── Storage Plugin (存储)
    └── Extras (扩展预留)
```

请求路径：
* 浏览器发起 API 请求
* Vite 开发代理转发到 `127.0.0.1:3001`
* Actix-web 路由到对应 handler
* handler 从 `AppState` 获取 `PluginRegistry`
* 调用具体的 Auth / Storage 插件

## 后端：6 个 Rust Crate

| Crate | 路径 | 职责 |
|-------|------|------|
| `ducia-core` | `backend/ducia-core/` | 核心 trait 定义（`AuthPlugin`、`DocRepository`），不包含任何具体实现 |
| `ducia-server` | `backend/server/` | 二进制入口，组装插件、加载配置、启动 HTTP 服务 |
| `ducia-auth-simple` | `backend/plugins/auth-simple/` | 序列码认证插件 |
| `ducia-auth-db` | `backend/plugins/auth-db/` | JWT + bcrypt + SQLite 认证插件 |
| `ducia-storage-fs` | `backend/plugins/storage-fs/` | 文件系统存储插件（JSON 索引 + .md 文件） |
| `ducia-storage-sqlite` | `backend/plugins/storage-sqlite/` | SQLite 存储插件 |

依赖关系：

- `ducia-core` 是**零依赖**（除 `serde` 等基础库），只定义 trait
- 4 个插件 crate 仅依赖 `ducia-core`，各自独立实现功能
- `ducia-server` 依赖所有 5 个 crate，在启动时按配置组装

## ducia-core 内部模块

```text
ducia-core/src/
├── lib.rs          # 公共 API 重新导出
├── doc/            # 文档模型
│   ├── model.rs    #   DocMeta, DocFull, CreateDocRequest, DocFormat
│   └── repo.rs     #   DocRepository trait
├── plugin/         # 插件系统
│   ├── auth.rs     #   AuthPlugin trait
│   ├── registry.rs #   PluginRegistry (builder pattern)
│   └── storage.rs  #   StoragePlugin = Box<dyn DocRepository>
├── perm/           # 权限引擎
│   └── model.rs    #   Identity, RoleDef, RoleConfig
└── i18n/           # 国际化
    └── mod.rs      #   I18nManager
```

lib.rs 负责重新导出所有公开类型，是 crate 的唯一入口。

## 插件注册表（PluginRegistry）

`PluginRegistry` 采用 **builder 模式**在启动时组装：

```rust
use ducia_core::plugin::registry::PluginRegistry;

let registry = PluginRegistry::new()
    .with_auth(auth_plugin)       // Arc<dyn AuthPlugin>
    .with_storage(storage_plugin); // Box<dyn DocRepository>
```

它持有：

- `auth: Option<Arc<dyn AuthPlugin>>` — 认证插件（框架约定只有一个活跃的认证插件）
- `storage: Option<StoragePlugin>` — 存储插件
- `extras: HashMap<String, Box<dyn Any + Send + Sync>>` — 未来扩展预留

整个生命周期内不变，线程安全（所有字段均为 `Send + Sync`）。

## AppState

`AppState` 是注入到 actix-web handler 中的全局状态：

```rust
pub struct AppState {
    pub plugins: PluginRegistry,       // 插件注册表
    pub role_config: Arc<RoleConfig>,  // 动态角色配置
    pub i18n: I18nManager,             // 国际化管理器
}
```

通过 actix-web 的 `web::Data<AppState>` 提取器注入：

```rust
pub async fn list_cats(state: web::Data<AppState>) -> impl Responder {
    let storage = state.plugins.storage().unwrap();
    let docs = storage.list_docs(false).await.unwrap();
    // ...
}
```

## 前端：React 18 + TypeScript

### 视图注册表（ViewRegistry）

前端不再硬编码 3 个视图，而是采用 **ViewRegistry** 模式，视图按 URL 模式动态注册和匹配：

- **`src/views/registry.ts`** — `ViewRegistry` 类，提供 `register()` 注册视图、`resolve()` 按当前路径匹配视图
- **`src/views/defaults.ts`** — 默认注册 3 个视图（listing、doc、admin）

当前已注册的默认视图及路径映射：

| 路径 | 视图组件 | 说明 |
|------|---------|------|
| `/` 或 `/listing` | `Listing` | 文档列表首页 |
| `/listing/lib/{id}` | `DocPage` | 文档详情/阅读页 |
| `/listing/lib/0` | `AdminPage` | 管理面板（序列码认证入口） |

> 以上 3 个视图仅为默认注册项。通过 `viewRegistry.register({name, match, component})` 即可添加新视图，路径映射完全可扩展。

### 核心 Hooks

| Hook | 文件 | 功能 |
|------|------|------|
| `useI18n` | `src/hooks/useI18n.tsx` | 国际化翻译（I18nProvider + useContext） |
| `useCats` | `src/hooks/useCats.ts` | 文档列表加载 |
| `useDoc` | `src/hooks/useDoc.ts` | 单文档获取 |
| `useAdmin` | `src/hooks/useAdmin.ts` | 序列码认证 + session 管理 |
| `useUpload` | `src/hooks/useUpload.ts` | 文档上传 |

### 前端入口

```typescript
// src/main.tsx
ReactDOM.createRoot(document.getElementById("root")!).render(
    <I18nProvider>
        <App />
    </I18nProvider>
);
```

`I18nProvider` 包裹整个应用，初始化时从 `/api/locales` 加载可用语言列表，根据浏览器 `Accept-Language` 解析初始语言。

## 开发代理

Vite 开发服务器（`:5173`）将所有 `/api/*` 请求代理到后端：

```js
// vite.config.js
export default defineConfig({
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:3001'
    }
  }
})
```

生产环境下，前后端文件一同打包到 `dist/` 目录，由 actix-web 通过 catch-all 路由 `/{tail:.*}` 托管：先尝试匹配精确静态文件，失败则返回 `index.html` 实现 SPA fallback。

## 服务启动流程

`ducia-server/src/main.rs` 按以下顺序启动：

1. 查找项目根目录（向上查找包含 `config/` 的目录）
2. 从 `config/roles.json` 加载角色配置
3. 从 `config/i18n/` 加载所有语言包，初始化 `I18nManager`
4. 根据 `config/settings.json` 的 `use_database` 字段选择存储后端
5. 根据 `config/auth.json` 是否存在选择认证后端（有则 `auth-db`，无则 `auth-simple`）
6. 通过 `PluginRegistry::new().with_auth(...).with_storage(...)` 组装插件
7. 创建 `AppState`，注入到 actix-web，绑定 `0.0.0.0:3001`
8. 注册所有 API 路由，启动 HTTP 服务
