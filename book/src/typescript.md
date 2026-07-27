# TypeScript 类型

Ducia 前端使用 TypeScript **strict mode**，所有组件和 Hook 均享有完整类型安全。类型定义集中在 `src/types/` 目录，按领域拆分。

## 类型文件概览

| 文件 | 领域 | 导出 |
|------|------|------|
| `doc.ts` | 文档数据模型 | `DocMeta`, `DocListItem`, `DocFull`, `CreateDocReq`, `DeprecatedReq`, `DocFormat`, `detectFormat()` |
| `api.ts` | API 请求/响应 | `ApiResponse<T>`, `ListCatsResponse`, `GetCatResponse`, `UploadCatResponse`, `SequenceResponse`, `CreateSessionResponse`, `CheckSessionResponse` |
| `auth.ts` | 身份认证 | `Permission`, `Identity`, `AuthPlugin`, `AuthState` |
| `plugin.ts` | 插件系统 | `Plugin`, `PluginKind`, `PluginRegistry` |
| `i18n.ts` | 国际化 | `TFunc`, `LocalePack`, `LocaleCode`, `I18nContext` |
| `index.ts` | 统一导出 | re-export 以上所有类型 |

---

## 文档模型 (`doc.ts`)

### DocMeta

文档在后端的完整元数据表示，对应存储层的记录：

```typescript
export interface DocMeta {
  title: string       // 文档标题
  file: string        // 存储文件名（相对 docs 目录）
  created_at: number  // 创建时间戳（ms since epoch）
  deprecated: boolean // 是否已弃用
  deleted: boolean    // 是否已软删除
}
```

### DocListItem

API 列表接口返回的精简视图，仅包含列表渲染所需字段：

```typescript
export interface DocListItem {
  id: string
  title: string
  created_at: number
}
```

### DocFull

文档完整数据，继承 `DocListItem` 并增加正文内容：

```typescript
export interface DocFull extends DocListItem {
  content: string    // Markdown 原文
  deprecated: boolean
}
```

### CreateDocReq / DeprecatedReq

请求载荷类型：

```typescript
export interface CreateDocReq {
  title: string
  content: string
}

export interface DeprecatedReq {
  deprecated: boolean
}
```

### DocFormat 与 detectFormat()

文档格式枚举和检测函数，用于区分 Markdown、纯文本、HTML：

```typescript
export type DocFormat = 'markdown' | 'plaintext' | 'html'

export function detectFormat(filename: string): DocFormat {
  const ext = filename.split('.').pop()?.toLowerCase()
  switch (ext) {
    case 'md': case 'markdown': return 'markdown'
    case 'txt': return 'plaintext'
    case 'html': case 'htm': return 'html'
    default: return 'markdown'
  }
}
```

该函数在 WASM 模块中有对应的 Rust 实现（`detect_format`），前后端使用相同逻辑。

---

## API 类型 (`api.ts`)

### 统一响应包裹

所有 API 响应均通过 `ApiResponse<T>` 统一包裹：

```typescript
export interface ApiResponse<T = unknown> {
  success: boolean
  data?: T
  message?: string
  siteName?: string
}
```

### 具体响应类型

| 类型 | 继承 | data 泛型 | 额外字段 |
|------|------|-----------|----------|
| `ListCatsResponse` | `ApiResponse<DocListItem[]>` | 文档列表 | `siteName` (必填) |
| `GetCatResponse` | `ApiResponse<DocFull>` | 单文档完整数据 | — |
| `UploadCatResponse` | `ApiResponse<DocListItem>` | 新上传的文档 | — |

### 管理/会话类型

```typescript
export interface SequenceResponse {
  sequence: number[]    // 服务端返回的目标序列
}

export interface CreateSessionResponse {
  token: string         // 认证令牌
}

export interface CheckSessionResponse {
  isAdmin: boolean      // session 是否有效且具有管理员身份
}
```

### 通用类型

```typescript
export interface SuccessResponse {
  success: boolean
}
```

---

## 认证类型 (`auth.ts`)

框架本体不预设角色系统，角色由部署者通过 `config/roles.json` 自定义。认证类型仅描述框架所需的最少信息。

### Permission / Identity

```typescript
export type Permission = string  // 格式: "resource:action"，如 "doc:read"

export interface Identity {
  id: string
  roles: string[]                     // 拥有的角色列表
  permissions: Permission[]           // 直接授予的权限
  metadata: Record<string, string>    // 插件可自由扩展的元数据
}
```

### AuthPlugin

认证插件的接口契约。每个认证插件需提供身份提取逻辑和前端 UI 组件：

```typescript
export interface AuthPlugin {
  name: string
  checkSession: () => Promise<Identity | null>
  destroySession: () => Promise<void>
  LoginComponent?: React.ComponentType<{
    onSuccess: (identity: Identity) => void
  }>
  AdminPanelComponent?: React.ComponentType
}
```

### AuthState

`useAuth` Hook 的返回值结构：

```typescript
export interface AuthState {
  identity: Identity | null
  loading: boolean
  can: (permission: Permission) => boolean
  hasRole: (role: string) => boolean
  createSession: (identity: Identity) => void
  destroySession: () => Promise<void>
}
```

---

## 插件类型 (`plugin.ts`)

前端插件注册表允许认证、存储、渲染等模块以插件形式接入。

### PluginKind / Plugin

```typescript
export type PluginKind = 'auth' | 'renderer' | 'upload' | 'panel'

export interface Plugin {
  id: string          // 唯一标识
  name: string        // 显示名称
  kind: PluginKind    // 插件类型
  version: string     // 版本号
}
```

### PluginRegistry

插件注册表的接口契约：

```typescript
export interface PluginRegistry {
  register(plugin: Plugin): void
  getByKind(kind: PluginKind): Plugin[]
  get(id: string): Plugin | undefined
  getAuthPlugin(): AuthPlugin | undefined
}
```

---

## 国际化类型 (`i18n.ts`)

翻译 key 采用点分隔的层级命名规范，如 `"doc.action.delete"`。所有面向用户的字符串均从语言包加载，代码中零硬编码。

### TFunc / LocalePack

```typescript
export type TFunc = (
  key: string,
  params?: Record<string, string | number>
) => string

export type LocalePack = Record<string, string>
```

翻译函数支持参数插值：

```typescript
t('doc.action.delete')                        // → "删除"
t('greeting', { name: 'Alice' })              // → "你好，Alice"
```

语言包中的模板使用 `{{key}}` 占位符：

```json
{
  "greeting": "你好，{{name}}"
}
```

### I18nContext

```typescript
export type LocaleCode = string

export interface I18nContext {
  locale: LocaleCode
  t: TFunc
  setLocale: (locale: LocaleCode) => void
  availableLocales: LocaleCode[]
}
```

---

## 统一导出 (`index.ts`)

所有类型通过 barrel export 集中导出：

```typescript
export * from './doc'
export * from './api'
export * from './auth'
export * from './plugin'
export * from './i18n'
```

组件和 Hook 中统一从 `'../types'` 导入，无需关心具体来源文件。
