# 变更日志

本文档记录 Ducia Listing Framework 的所有重要版本变更。

---

## v0.3.0 (开发中)

### 新增

- **mdBook 文档系统**：完整的中文开发手册，涵盖架构、配置、API、开发指南
- **国际化系统 (i18n)**：前端 `useI18n` Hook + JSON 语言包，支持请求级语言解析，翻译 key 点分隔层级命名，代码中零硬编码字符串
- **auth-db 认证插件**：基于 JWT + bcrypt 的用户认证，支持注册/登录/Token 刷新，用户数据可存储于文件系统或 SQLite
- **storage-sqlite 存储插件**：基于 rusqlite 的 SQLite 文档存储，支持索引、搜索、软删除与弃用标记
- **WASM 共享逻辑**：将 Markdown 渲染（pulldown-cmark）和序列码验证逻辑编译为 WebAssembly，前后端共享同一套 Rust 代码
- **视图注册表（ViewRegistry）**：页面不再写死为 3 个，通过 URL 模式匹配动态注册
- **格式注册表（FormatRegistry）**：Markdown 只是默认注册的格式之一，上传/渲染均通过注册表查找
- **配置热切换**：通过 `config/settings.json` 可在文件存储 ↔ SQLite 存储之间切换，无需重新编译
- **动态角色系统**：`config/roles.json` 定义角色与权限映射，框架不预设 admin/editor/viewer

### 改进

- 前端所有用户可见字符串改为从语言包加载
- 文档列表项增加鼠标悬停预加载缓存
- 顶栏吸顶效果优化（`useStickyHeader` Hook）
- 删除操作增加二次确认机制
- 文档页增加弃用/恢复操作

### 变更

- 重构后端为 workspace 结构：`ducia-core` 核心库 + 多个插件 crate
- API 响应统一为 `ApiResponse<T>` 格式

---

## v0.3.1 (2026-06)

### 修复

- **Docker 构建**：Rust 基础镜像升级到 `rust:1.88-alpine`，解决依赖版本不兼容问题
- **SPA 路由**：新增自定义 catch-all handler `/{tail:.*}`，解决深层路径（如 `/listing/lib/0`）返回 404 的问题
- **存储写入**：`FsStorage` 的 `docs.json` 从只读的 `config/` 移至可写的 `data/`，修复弃用/上传/删除操作返回 500 的问题；首次启动自动迁移已有数据
- **视图路由**：修正 `admin` 视图注册顺序（置于 `doc` 之前），修复 `/listing/lib/0` 被 `doc` 视图抢先匹配而跳回首页的问题
- **认证界面样式**：`AdminPage` 复用 `MenuBar.module.css` 样式，修复全局 CSS 类名缺失导致的渲染异常
- **国际化**：`useI18n` 语言包加载添加 `.catch()` 错误处理，消除未捕获的 Promise 拒绝

---

## v0.2.0

### 新增

- **插件架构 (ducia-core)**：定义核心 trait（`DocStorage`、`AuthProvider`），存储和认证通过插件注入
- **动态角色系统**：角色与权限通过 `config/roles.json` 配置，不硬编码
- **外部认证接口**：支持序列码认证（内置）和外部认证服务（可扩展）
- **文件系统存储插件 (storage-fs)**：基于文件系统的文档存储，`docs/` 目录直接映射
- **多格式文档支持**：Markdown / 纯文本 / HTML 文档格式检测与渲染

### 改进

- 后端从 monolithic 拆分为 core + plugins 结构
- 前端路由从硬编码改为路径分发
- 文档列表支持创建时间排序

---

## v0.1.0

### 初始发布

- **后端**：基于 Actix-web 4 的 REST API 服务
  - `GET /api/cats` — 文档列表
  - `GET /api/cats/{id}` — 获取文档
  - `POST /api/cats` — 上传文档
  - `PUT /api/cats/{id}/deprecated` — 标记弃用
  - `PUT /api/cats/{id}/deleted` — 软删除
  - `POST /api/admin/session` — 创建管理会话
  - `GET /api/admin/session` — 检查会话状态
  - `DELETE /api/admin/session` — 销毁会话
- **前端**：React 18 + TypeScript + Vite 单页应用
  - 文档列表页（`/`）
  - 文档阅读页（`/listing/lib/{id}`）
  - 管理认证页（`/listing/lib/0`）
  - Markdown 渲染（react-markdown + remark-gfm + remark-math + rehype-katex）
- **文件存储**：文档以 Markdown 文件形式存储于 `docs/` 目录
- **序列码认证**：通过点击数字按钮输入预设序列，成功后获得管理员权限
- **基本管理功能**：上传 `.md` 文件、标记弃用、软删除
- **CORS 支持**：actix-cors 中间件

---

## 版本规范

Ducia 遵循 [语义化版本](https://semver.org/lang/zh-CN/)：

- **主版本号 (MAJOR)**：API 不兼容变更
- **次版本号 (MINOR)**：新增向后兼容功能
- **修订号 (PATCH)**：向后兼容的问题修复

版本号在以下文件中同步更新：

| 文件 | 字段 |
|------|------|
| `backend/server/Cargo.toml` | `version` |
| `backend/ducia-core/Cargo.toml` | `version` |
