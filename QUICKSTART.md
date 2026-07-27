# 快速启动

## 方式一：Docker（推荐，零依赖）

```bash
# 构建并启动
docker compose up -d

# 浏览器打开 http://localhost:3001

# 停止
docker compose down
```

配置文件、文档、数据通过 volume 挂载到宿主机：
- `config/` → 修改配置后 `docker compose restart` 生效
- `docs/` → 上传的文档保存在这里
- `data/` → SQLite 数据库文件

## 方式二：本地开发

```bash
# 1. 安装依赖（首次）
npm install

# 2. 启动开发模式（后端 + 前端一起）
./scripts/run-dev.sh

# 3. 浏览器打开 http://localhost:5173

# 4. 停止
./scripts/stop-dev.sh
```

# 管理员登录

1. 在首页点击左下角 Home 图标 → 进入管理面板
2. 按照 config/sequence.json 中的序列点击按钮：1,2,3,2,3,4
3. 成功后自动获得管理员权限

# 或使用用户注册

如果 config/auth.json 存在（默认存在）：
- POST /api/auth/register 注册
- POST /api/auth/login 登录
- 返回 JWT token，后续请求带 Authorization: Bearer <token>

# 配置切换

- `config/settings.json` → `"use_database": true` 启用 SQLite 存储
- 删除 `config/auth.json` → 回退到序列码认证
- `config/roles.json` → 自定义角色权限
