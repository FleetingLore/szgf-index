# API 参考

全部 API 均以 JSON 格式交互，基础路径为 `http://127.0.0.1:3001`。

## 通用约定

### 响应格式

所有接口返回统一 JSON 结构：

```json
{
  "success": true,
  "data": { ... },
  "message": "错误描述（仅失败时）"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `success` | `bool` | 操作是否成功 |
| `data` | `object` / `array` / `null` | 业务数据载荷 |
| `message` | `string`（可选） | 失败时的错误描述 |

### 认证方式

不同认证插件使用不同的请求头：

| 插件 | 请求头 | 格式 |
|------|--------|------|
| `auth-simple` | `X-Session-Token` | `X-Session-Token: <uuid>` |
| `auth-db` | `Authorization` | `Authorization: Bearer <jwt>` |

---

## 文档 API

### `GET /api/cats` — 列出文档

获取所有可见文档（不含已删除和已弃用的文档）。

**无需认证。**

**请求示例**

```bash
curl http://127.0.0.1:3001/api/cats
```

**响应示例**

```json
{
  "success": true,
  "data": [
    {
      "id": "1",
      "title": "示例文档",
      "created_at": 1782165910244
    }
  ],
  "siteName": "25 电科 3 班待办事项清单"
}
```

**响应字段**

| 字段 | 类型 | 说明 |
|------|------|------|
| `success` | `bool` | 是否成功 |
| `data` | `array` | 文档列表 |
| `data[].id` | `string` | 文档唯一 ID |
| `data[].title` | `string` | 文档标题 |
| `data[].created_at` | `number` | 创建时间（毫秒时间戳） |
| `siteName` | `string` | 站点名称 |

---

### `GET /api/cats/{id}` — 获取单篇文档

获取指定文档的完整内容。

**无需认证。**

**路径参数**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | `string` | 文档 ID |

**请求示例**

```bash
curl http://127.0.0.1:3001/api/cats/1
```

**响应示例**

```json
{
  "success": true,
  "data": {
    "id": "1",
    "title": "示例文档",
    "content": "# Hello Ducia\n\n这是示例内容。",
    "created_at": 1782165910244,
    "deprecated": false
  }
}
```

**响应字段**

| 字段 | 类型 | 说明 |
|------|------|------|
| `data.id` | `string` | 文档 ID |
| `data.title` | `string` | 文档标题 |
| `data.content` | `string` | Markdown 原始内容 |
| `data.created_at` | `number` | 创建时间（毫秒时间戳） |
| `data.deprecated` | `bool` | 是否已弃用 |

**错误响应**

```json
{ "success": false, "message": "not found" }
```

---

### `POST /api/cats` — 创建文档

上传一篇新文档。

**需要认证**（`doc:write` 权限）。

**请求体**

```json
{
  "title": "新文档标题",
  "content": "# Markdown 内容"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `title` | `string` | 是 | 文档标题 |
| `content` | `string` | 是 | Markdown 格式的文档正文 |

**请求示例**

```bash
curl -X POST http://127.0.0.1:3001/api/cats \
  -H 'Content-Type: application/json' \
  -H 'X-Session-Token: <admin-token>' \
  -d '{"title":"新文档","content":"# 新文档\n内容..."}'
```

**响应示例**

```json
{
  "success": true,
  "data": {
    "id": "2",
    "title": "新文档",
    "created_at": 1782166000000
  }
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `data.id` | `string` | 新创建的文档 ID |
| `data.title` | `string` | 文档标题 |
| `data.created_at` | `number` | 创建时间（毫秒时间戳） |

---

### `PUT /api/cats/{id}/deprecated` — 切换弃用状态

将文档标记为弃用，或取消弃用标记。

**需要认证**（`doc:deprecate` 权限）。

**路径参数**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | `string` | 文档 ID |

**请求体**

```json
{
  "deprecated": true
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `deprecated` | `bool` | 是 | `true` 标记弃用，`false` 取消弃用 |

**请求示例**

```bash
curl -X PUT http://127.0.0.1:3001/api/cats/1/deprecated \
  -H 'Content-Type: application/json' \
  -H 'X-Session-Token: <admin-token>' \
  -d '{"deprecated":true}'
```

**响应示例**

```json
{ "success": true }
```

---

### `PUT /api/cats/{id}/deleted` — 软删除文档

将文档标记为已删除（软删除，不删除实际文件）。

**需要认证**（`doc:delete` 权限）。

**路径参数**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | `string` | 文档 ID |

**请求示例**

```bash
curl -X PUT http://127.0.0.1:3001/api/cats/1/deleted \
  -H 'X-Session-Token: <admin-token>'
```

**响应示例**

```json
{ "success": true }
```

---

### `PUT /api/cats/{id}/lock` — 锁定/解锁文档

锁定后非管理员无法弃用或恢复文档。已弃用且锁定的文档不可恢复，但可以删除。

**需要认证**（管理员）。

**路径参数**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | `string` | 文档 ID |

**请求体**

```json
{
  "locked": true
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `locked` | `bool` | 是 | `true` 锁定，`false` 解锁 |

**请求示例**

```bash
curl -X PUT http://127.0.0.1:3001/api/cats/1/lock \
  -H 'Content-Type: application/json' \
  -H 'X-Session-Token: <admin-token>' \
  -d '{"locked":true}'
```

**响应示例**

```json
{ "success": true }
```

---

## 管理 API

### `GET /api/admin/sequence` — 获取登录序列

返回 `config/sequence.json` 中定义的管理员登录按钮序列。

**无需认证。**

**请求示例**

```bash
curl http://127.0.0.1:3001/api/admin/sequence
```

**响应示例**

```json
{ "sequence": [1, 2, 3, 2, 3, 4] }
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `sequence` | `number[]` | 按钮点击序列 |

---

### `POST /api/admin/session` — 创建管理员会话

验证序列码正确后，创建管理员会话并返回 token。

**无需认证**（但前端需提供正确的序列码）。

**请求示例**

```bash
curl -X POST http://127.0.0.1:3001/api/admin/session
```

**响应示例**

```json
{ "token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890" }
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `token` | `string` | 会话令牌 |

> **说明**：`auth-simple` 模式返回 UUID token；`auth-db` 模式返回 JWT token（有效期 24 小时）。

---

### `GET /api/admin/session` — 检查会话

检查当前会话是否有效（是否为管理员）。

**请求头**

```
X-Session-Token: <token>
```

**请求示例**

```bash
curl http://127.0.0.1:3001/api/admin/session \
  -H 'X-Session-Token: a1b2c3d4-e5f6-7890-abcd-ef1234567890'
```

**响应示例**

```json
{ "isAdmin": true }
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `isAdmin` | `bool` | 当前会话是否为管理员 |

---

### `DELETE /api/admin/session` — 销毁会话

登出管理员，销毁当前会话。

**请求头**

```
X-Session-Token: <token>
```

**请求示例**

```bash
curl -X DELETE http://127.0.0.1:3001/api/admin/session \
  -H 'X-Session-Token: a1b2c3d4-e5f6-7890-abcd-ef1234567890'
```

**响应示例**

```json
{ "success": true }
```

---

## 认证 API

> 以下接口仅在 `auth-db` 模式下可用（`config/auth.json` 存在时）。

### `POST /api/auth/register` — 用户注册

注册新用户，默认赋予 `viewer` 角色。

**无需认证。**

**请求体**

```json
{
  "username": "alice",
  "password": "1234"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `username` | `string` | 是 | 用户名，不可为空 |
| `password` | `string` | 是 | 密码，至少 4 个字符 |

**请求示例**

```bash
curl -X POST http://127.0.0.1:3001/api/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"1234"}'
```

**响应示例**

```json
{
  "success": true,
  "data": {
    "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    "username": "alice",
    "roles": ["viewer"]
  }
}
```

**响应字段**

| 字段 | 类型 | 说明 |
|------|------|------|
| `data.id` | `string` | 用户唯一 ID（UUID v4） |
| `data.username` | `string` | 用户名 |
| `data.roles` | `string[]` | 角色列表 |

---

### `POST /api/auth/login` — 用户登录

验证用户名密码，返回 JWT token。

**无需认证。**

**请求体**

```json
{
  "username": "alice",
  "password": "1234"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `username` | `string` | 是 | 用户名 |
| `password` | `string` | 是 | 密码 |

**请求示例**

```bash
curl -X POST http://127.0.0.1:3001/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"1234"}'
```

**响应示例**

```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIs...",
    "user": {
      "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
      "username": "alice",
      "roles": ["viewer"]
    }
  }
}
```

**响应字段**

| 字段 | 类型 | 说明 |
|------|------|------|
| `data.token` | `string` | JWT 访问令牌（7 天有效） |
| `data.user.id` | `string` | 用户 ID |
| `data.user.username` | `string` | 用户名 |
| `data.user.roles` | `string[]` | 用户角色列表 |

**错误响应**（401）

```json
{ "success": false, "message": "invalid username or password" }
```

---

### `GET /api/auth/me` — 当前用户信息

使用 JWT token 获取当前登录用户的身份信息。

**需要认证。**

**请求头**

```
Authorization: Bearer <jwt-token>
```

**请求示例**

```bash
curl http://127.0.0.1:3001/api/auth/me \
  -H 'Authorization: Bearer eyJhbGciOiJIUzI1NiIs...'
```

**响应示例**

```json
{
  "success": true,
  "data": {
    "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    "username": "alice",
    "roles": ["viewer"]
  }
}
```

**错误响应**（401）

```json
{ "success": false, "message": "unauthorized" }
```

---

## 国际化 API

### `GET /api/locales` — 可用语言列表

返回所有已安装的语言包列表和默认语言。

**无需认证。**

**请求示例**

```bash
curl http://127.0.0.1:3001/api/locales
```

**响应示例**

```json
{
  "success": true,
  "data": [
    { "code": "zh-CN", "name": "简体中文", "dir": "ltr" },
    { "code": "en", "name": "English", "dir": "ltr" }
  ],
  "default": "zh-CN"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `data` | `array` | 语言列表 |
| `data[].code` | `string` | 语言代码（locale） |
| `data[].name` | `string` | 语言显示名称 |
| `data[].dir` | `string` | 文字方向 |
| `default` | `string` | 默认语言代码 |

---

### `GET /api/locales/{locale}` — 获取翻译包

获取指定语言的完整翻译内容。

**无需认证。**

**路径参数**

| 参数 | 类型 | 说明 |
|------|------|------|
| `locale` | `string` | 语言代码，如 `zh-CN`、`en` |

**请求示例**

```bash
curl http://127.0.0.1:3001/api/locales/zh-CN
```

**响应示例**

```json
{
  "success": true,
  "data": {
    "@meta.name": "简体中文",
    "@meta.dir": "ltr",
    "site.title": "25 电科 3 班待办事项清单",
    "doc.status.loading": "加载中...",
    "doc.action.upload": "上传"
  }
}
```

---

## API 速查表

| 方法 | 路径 | 认证 | 权限 | 说明 |
|------|------|------|------|------|
| `GET` | `/api/cats` | 否 | - | 列出文档 |
| `GET` | `/api/cats/{id}` | 否 | - | 获取单篇文档 |
| `POST` | `/api/cats` | 是 | `doc:write` | 创建文档 |
| `PUT` | `/api/cats/{id}/deprecated` | 是 | `doc:deprecate` | 切换弃用 |
| `PUT` | `/api/cats/{id}/deleted` | 是 | `doc:delete` | 软删除 |
| `PUT` | `/api/cats/{id}/lock` | 是（管理员） | - | 锁定/解锁 |
| `GET` | `/api/admin/sequence` | 否 | - | 获取登录序列 |
| `POST` | `/api/admin/session` | 否 | - | 创建管理员会话 |
| `GET` | `/api/admin/session` | 否 | - | 检查会话 |
| `DELETE` | `/api/admin/session` | 否 | - | 销毁会话 |
| `POST` | `/api/auth/register` | 否 | - | 用户注册 |
| `POST` | `/api/auth/login` | 否 | - | 用户登录 |
| `GET` | `/api/auth/me` | 是 | - | 当前用户信息 |
| `GET` | `/api/locales` | 否 | - | 语言列表 |
| `GET` | `/api/locales/{locale}` | 否 | - | 翻译包 |

> **注**：`/api/auth/*` 系列接口仅在 `auth-db` 模式下可用（`config/auth.json` 存在时）。
