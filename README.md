# ducia-listing-framework — Fullstack Docs Template

这是一个包含 Rust 后端（Actix-web）和前端（Vite + React）的示例工程，已准备为 GitHub 模板仓库与可发布的 Rust crate（后端）使用。

使用为 GitHub 模板

- 在 GitHub 上选择仓库设置并启用 "Template repository"（Enable as template）。
- 点击 "Use this template" 可以基于本仓库快速创建新项目。

发布到 GitHub 并用 Rust 文档作为站点

1. 如果你想把仓库发布到 GitHub（建议先在本地确认一切工作正常）：

	 - 首先安装并登录 GitHub CLI（可选，但推荐）：

		 ```bash
		 gh auth login
		 ```

	 - 使用我们的辅助脚本来创建仓库并推送：

		 ```bash
		 ./scripts/init_github_repo.sh
		 ```

	 - 脚本会在 `FleetingLore/ducia-listing-framework` 下创建仓库（需要你的 GitHub 账户有对应权限），并把当前代码推到 `main` 分支。

2. GitHub Actions 已配置为在每次 push 到 `main` 时构建 `backend` 的 Rust 文档并将 `backend/target/doc` 发布到 `gh-pages` 分支：

	 - 工作流文件： [.github/workflows/publish-docs.yml](.github/workflows/publish-docs.yml#L1)
	 - CI 使用 `cargo doc` 生成文档并通过 `peaceiris/actions-gh-pages` 发布到 `gh-pages`。

3. 在仓库的 GitHub 页面，打开 Settings → Pages，设置 Source 为 `gh-pages` 分支（工作流运行后会自动创建并推送内容）。


包含内容

- `backend/`：Rust 后端（可作为库发布到 crates.io，或以二进制运行）。
- `src/` 与 `public/`：前端代码（Vite + React）。
- `config/`、`docs/`：默认配置与示例文档。

发布后端到 crates.io（要点）

1. 在 `backend/Cargo.toml` 中填写 `description`、`license`、`readme`、`repository` 等元数据。
2. 在 `backend/` 目录下运行 `cargo publish`（需要有效的 crates.io token）。

注意：GitHub 模板的开关需要在 GitHub 界面上手动启用。

## 可复现的开发环境（一键准备）

为了让其他人在他们的设备上更容易复现本项目，我们提供了一组集中化的脚本和配置：


使用建议：

1. 复制并编辑部署配置：

	```bash
	cp scripts/deploy.conf.example scripts/deploy.conf
	# 编辑 scripts/deploy.conf 中的 SERVER / REMOTE_USER / REMOTE_PATH 等项
	```

2. 本地环境检查（并在 macOS 下安装缺失项）：

	```bash
	scripts/setup_dev_env.sh        # 检查
	scripts/setup_dev_env.sh --install   # macOS 上自动尝试安装（需要 Homebrew）
	```

3. 安装 git hooks（可选）：

	```bash
	bash scripts/git-hooks/install.sh
	```

4. 常用命令：

	- 构建前端：`make build-frontend`
	- 构建后端：`make build-backend`
	- 生成并发布文档：`make publish-docs`
	- 部署到服务器：`make deploy` 或 `./deploy.sh`

## 本地运行（快速开始）

如果你只是想在本机快速运行并修改代码：

1. 检查并安装依赖：

```bash
./scripts/setup_dev_env.sh
# (macOS) ./scripts/setup_dev_env.sh --install
```

2. 安装前端依赖并启动开发模式：

```bash
npm install
./scripts/run-dev.sh       # 一键启动后端+前端
./scripts/stop-dev.sh      # 停止
```

3. Docker 部署（无需安装 Rust/Node）：

```bash
docker compose up -d
# 访问 http://localhost:3001
```


## 在 GitHub Actions 中自动发布（可选）

三个工作流文件在 `.github/workflows/`：

| 文件 | 用途 |
|------|------|
| `ci.yml` | push 时编译检查 + Docker 构建验证 + 手动确认|
| `pages.yml` | mdBook 文档自动部署到 GitHub Pages |
| `publish.yml` | 推送版本 tag 时自动发布到 crates.io |

其中 `publish.yml` 需要在 Settings → Secrets 添加：
- `CRATES_IO_TOKEN` — 你的 crates.io API token

版本号统一在 `backend/Cargo.toml` 的 `[workspace.package]` 中管理，所有 crate 通过 `version.workspace = true` 继承。发布流程：

```bash
# 改 backend/Cargo.toml 里的 version，然后
git tag v0.3.0
git push origin main
git push origin v0.3.0
```

详见[开发手册 - CI/CD 工作流](https://fleetinglore.github.io/ducia-listing-framework/ci-cd.html)。
