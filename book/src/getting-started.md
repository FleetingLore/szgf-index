# 快速开始

## 环境要求

- Rust (1.70+)
- Node.js (18+)
- npm

## 一键启动

```bash
# 安装前端依赖
npm install

# 启动（后端 + 前端开发服务器）
./scripts/run-dev.sh

# 浏览器打开
open http://localhost:5173

# 停止
./scripts/stop-dev.sh
```

## 管理员登录

首页左下角点击 Home 图标进入管理面板，按序列点击按钮：

```
sequence.json 中定义: [1, 2, 3, 2, 3, 4]
```

成功后自动获得管理员权限，可以上传/删除/弃用文档。

## 用户注册（需要 auth-db）

如果 `config/auth.json` 存在：

```bash
# 注册
curl -X POST http://127.0.0.1:3001/api/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"1234"}'

# 登录
curl -X POST http://127.0.0.1:3001/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"1234"}'
# 返回 {"token": "eyJ...", "user": {...}}

# 使用 token
curl http://127.0.0.1:3001/api/auth/me \
  -H 'Authorization: Bearer eyJ...'
```

## 配置切换

| 配置 | 文件 | 操作 |
|------|------|------|
| 启用 SQLite 存储 | `config/settings.json` | `"use_database": true` |
| 回退序列码认证 | - | 删除 `config/auth.json` |
| 自定义角色 | `config/roles.json` | 编辑角色权限 |
| 添加语言 | `config/i18n/` | 新增 `{locale}.json` |
