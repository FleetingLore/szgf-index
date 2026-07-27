# CI / CD 工作流

项目通过 GitHub Actions 实现自动化构建、文档部署和 crates.io 发布。所有工作流定义在 `.github/workflows/` 目录下。

---

## 工作流总览

| 文件 | 触发条件 | 做什么 |
|------|----------|--------|
| `ci.yml` | push main/indev、PR、手动 | 编译检查 → Docker 构建 → 手动确认部署 |
| `pages.yml` | push main/indev（仅 `book/` 变更）、手动 | mdBook 构建 → 部署到 GitHub Pages |
| `publish.yml` | 推送版本 tag（如 `v0.2.0`）、手动 | 按顺序发布 crate 到 crates.io |

---

## CI（ci.yml）

```
push / PR / 手动触发
  │
  ├── [1] backend   cargo check（后端编译检查）
  ├── [2] frontend  npm ci + npm run build（前端构建检查）
  ├── [3] docker    docker compose build（镜像构建验证）
  └── [4] approve   🛑 手动确认（仅 main/indev 分支 push 时触发）
```

**手动确认机制**：第 4 步 `approve-deploy` 使用 `environment: production`。在 GitHub 仓库 Settings → Environments 中创建 `production` 环境，勾选 **Required reviewers**，每次部署前需人工点 Approve。

也可在 Actions 页面手动触发（`workflow_dispatch`），跳过 push 条件。

### 运行测试

CI 目前只做编译检查，不含单元测试。如需添加测试步骤，在 `backend` job 中追加：

```yaml
- name: cargo test
  run: cargo test --workspace
  working-directory: backend/server
```

---

## Pages（pages.yml）

```
push book/** 或 手动触发
  │
  ├── [1] build    下载 mdBook → mdbook build
  │                upload artifact（HTML 不进仓库）
  └── [2] deploy   deploy-pages（GitHub 原生）
```

**特点**：HTML 产物不在仓库中存储，由 Actions 临时构建并部署。不再需要 `gh-pages` 分支。

### 首次启用

GitHub 仓库 → Settings → Pages → Source 选 **GitHub Actions**。

---

## Publish（publish.yml）

```
git tag v0.2.0 && git push origin v0.2.0
  │
  ├── [1] publish-core    先发 ducia-core（插件依赖它）
  └── [2] publish-plugins 并发发布 4 个插件（等 core 成功后）
```

### 前置准备

1. 在 https://crates.io/settings/tokens 生成 API token
2. GitHub 仓库 → Settings → Secrets → Actions → 新建 `CRATES_IO_TOKEN`，粘贴 token

### 发布流程

```bash
# 1. 修改 backend/Cargo.toml 中的 workspace.package.version
#    所有 crate 通过 version.workspace = true 继承此版本号

# 2. 提交并推送
git add -A
git commit -m "release: v0.3.0"

# 3. 打 tag 推送（触发自动发布）
git tag v0.3.0
git push origin main
git push origin v0.3.0
```

### 版本号管理

项目使用 Cargo workspace 统一管理版本号。**修改版本只需改一处**：

```toml
# backend/Cargo.toml（唯一需要改的地方）
[workspace.package]
version = "0.3.0"       # ← 这里
edition = "2024"
license = "MIT"
repository = "https://github.com/FleetingLore/ducia-listing-framework"
```

所有子 crate 通过 `version.workspace = true` 自动继承。`changelog.md` 和文档中的版本号需手动同步。

### 手动触发

除 tag 推送外，也可在 Actions 页面 → Publish → **Run workflow** 手动触发。

---

## 加密变量

需要在 GitHub 仓库 Settings → Secrets and variables → Actions 中配置：

| Secret | 用途 | 哪个工作流用 |
|--------|------|-------------|
| `CRATES_IO_TOKEN` | crates.io API token | `publish.yml` |

CI 和 Pages 工作流无需额外 secret。

---

## 本地运行工作流

使用 [act](https://github.com/nektos/act) 可在本地模拟运行：

```bash
# 安装
brew install act

# 模拟 push 事件
act push

# 模拟特定 job
act -j backend
```

> `act` 需要 Docker，且部分 step（如 `actions/deploy-pages`）仅 GitHub 环境支持。
