# 开发环境

本章介绍如何搭建 Ducia 的本地开发环境、运行测试以及项目目录结构。

---

## 环境要求

| 工具 | 最低版本 | 说明 |
|------|----------|------|
| Rust | 1.70+ | 后端编译、运行 |
| Node.js | 18+ | 前端构建、开发服务器 |
| npm | 随 Node.js | 包管理 |
| wasm-pack | 可选 | 仅修改 WASM 模块时需要 |

### 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 安装 Node.js

推荐使用 [nvm](https://github.com/nvm-sh/nvm) 管理版本。项目根目录有 `.nvmrc` 指定推荐版本：

```bash
nvm install
nvm use
```

### 安装 wasm-pack（可选）

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

---

## 一键启动

项目提供开发脚本，同时启动后端和前端：

```bash
./scripts/run-dev.sh
```

该脚本完成以下步骤：

1. 检查 `cargo` 和 `npm` 是否可用
2. 如果 `docs/` 为空，创建示例文档 `docs/welcome.md`
3. 如果 `node_modules/` 不存在，自动执行 `npm install`
4. 启动后端：`cargo run -p ducia-server`（后台运行，日志输出到 `logs/backend.log`）
5. 等待后端就绪（探测 `http://127.0.0.1:3001/api/locales`）
6. 启动前端：`npm run dev`

> 按 `Ctrl+C` 停止时，脚本会通过 trap 自动清理后端进程。浏览器访问 `http://localhost:5173`。

### 停止开发服务

```bash
./scripts/stop-dev.sh
```

---

## Docker 部署

如果不想安装 Rust / Node.js，直接用 Docker：

```bash
docker compose up -d
# 浏览器打开 http://localhost:3001
```

`Dockerfile` 分三个阶段：

1. **frontend**：`node:20-alpine` 构建 `npm run build` → `dist/`
2. **backend**：`rust:1.88-alpine` 编译 `cargo build --release` → 二进制
3. **runtime**：`alpine:3.20` 仅复制产物，镜像体积最小

`docker-compose.yml` 将 `config/`、`docs/`、`data/` 挂载到宿主机：修改配置文件后 `docker compose restart` 生效；上传的文档保留在 `docs/`，容器重建不丢失。

生产环境下后端通过 catch-all 路由 `/{tail:.*}` 同时 serve 前端静态文件（先匹配精确文件，失败返回 `index.html`）。

停止：

```bash
docker compose down
```

---

## 分步启动

### 启动后端

```bash
cd backend/server
cargo run
```

后端监听 `http://127.0.0.1:3001`。首次运行会创建 `config/`、`data/` 目录并自动迁移数据库表。

### 启动前端

```bash
npm run dev
```

Vite 开发服务器监听 `http://localhost:5173`，API 请求自动代理到后端。

---

## 快速重建（修改代码后）

```bash
npm run build
cd backend/server && cargo test && cargo build --release && cd ../..
lsof -ti :3001 | xargs kill -9
DUCIA_DEBUG=true ./backend/target/release/ducia-server &
```

> ⚠️ 后端二进制在 `backend/target/release/`（workspace 根），不是 `backend/server/target/`。`lsof -ti :3001` 按端口杀进程比 `pkill` 更可靠。重启后 `curl -s http://localhost:3001/ | grep "index.*js"` 确认 hash 变了——没变是浏览器缓存，Cmd+Shift+R 硬刷新。

---

## 调试模式

环境变量 `DUCIA_DEBUG=true` 同时控制前后端日志，**无需改 URL 或浏览器设置**：

```bash
DUCIA_DEBUG=true ./backend/target/release/ducia-server &    # 调试
./backend/target/release/ducia-server &                      # 正常（生产）
```

| 模式 | 终端输出 | 浏览器控制台 |
|------|---------|------------|
| `DUCIA_DEBUG=true` | 每条 HTTP 请求的 method + path + 状态码 | `[debug] DocFooter isAdmin: ...` 组件状态 |
| 默认 | 仅启动信息 | 无输出 |

**原理**：服务端启动时读 `DUCIA_DEBUG`，若为 `true` 则在 `index.html` 中注入 `<script>window.__DUCIA_DEBUG=true</script>`。前端 `debugLog()` 检查此标记。不开启时零开销。

---

## 后端测试

```bash
cd backend/server && cargo test
```

当前 6 个测试用例（创建、弃用/恢复、锁定、软删除）。测试失败则后续 `cargo build --release` 不会执行。新增功能应同步补测试。

---

## WASM 构建

仅修改 `backend/wasm/src/lib.rs` 时才需要：

```bash
cd backend/wasm
wasm-pack build --target web
cp pkg/*.js ../../src/wasm/
cp pkg/*.wasm ../../public/
cp pkg/*.d.ts ../../src/wasm/
```

详见 [WASM 构建](./wasm.md)。

---

## 分支工作流

新功能开发遵循 Feature Branch 模式：

```
main ←── indev ←── feature/xxx
  │        │           │
  │        │    开发 + 本地测试
  │        │           │
  │        │    merge 回 indev → 服务器 Docker 验证
  │        │
  │    merge 回 main → 打 tag 发布
```

### 操作步骤

```bash
# 1. 从 indev 开分支
git checkout indev
git checkout -b feature/xxx

# 2. 开发 + 测试
cargo test && npm run build

# 3. 合回 indev
git checkout indev
git merge feature/xxx
git push origin indev

# 4. indev 跑稳后合回 main
git checkout main
git merge indev
git tag v0.3.0
git push origin main --tags
```

原则：一个分支一个功能。`cargo test` + `npm run build` 通过才能 merge。

---

## 脚本参考

| 脚本 | 用途 | 何时用 |
|------|------|--------|
| `run-dev.sh` | 一键启动后端 + 前端开发服务器 | 日常开发 |
| `stop-dev.sh` | 停止 `run-dev.sh` 启动的后端进程 | 开发结束 |
| `setup_dev_env.sh` | 检查 Rust/Node 等工具；加 `--install` 自动装（macOS） | 首次搭建环境 |
| `git-hooks/install.sh` | 安装 git pre-push hook | 可选 |
| `git-hooks/pre-push` | push 前自动编译检查 | 由 hook 触发 |

---

## 项目布局

```text
ducia-listing-framework/
├── config/          # 运行时配置
├── backend/         # 后端
│   ├── ducia-core/  # 框架核心
│   ├── plugins/     # 插件
│   ├── server/      # 入口
│   └── wasm/        # WASM
├── src/             # 前端源码
├── public/          # 静态资源
├── scripts/         # 脚本
├── book/            # 开发手册
└── data/            # 运行时数据
```

| 路径 | 用途 |
|------|------|
| `config/` | 所有配置文件，修改后重启后端生效 |
| `docs/` | Markdown 文档存放目录，上传的文档写入此处 |
| `data/` | SQLite 数据库文件，`storage-sqlite` 使用 |

---

## 配置切换

| 场景 | 操作 |
|------|------|
| 启用 SQLite 存储 | `config/settings.json` 设 `"use_database": true` |
| 回退序列码认证 | 删除 `config/auth.json` |
| 自定义角色权限 | 编辑 `config/roles.json` |
| 添加新语言 | `config/i18n/` 中新增 `{locale}.json` |

---

## 故障排查

### 后端启动失败

```bash
cat logs/backend.log
# 常见原因：端口 3001 被占用、config 权限不足、Rust 版本过旧
lsof -i :3001
```

### 前端无法连接后端

```bash
curl http://127.0.0.1:3001/api/locales
```

### TypeScript 类型错误

```bash
rm -rf node_modules package-lock.json && npm install && npx tsc --noEmit
```
