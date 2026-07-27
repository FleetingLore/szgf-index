# ducia-listing-framework (backend)

`ducia-listing-framework` 是一个使用 Rust + Actix-web 编写的轻量级文档服务器后端。

快速开始

- 本地运行：

```
cd backend
cargo run --release
```

- 作为库使用：

在另外一个 Rust 项目的 `Cargo.toml` 中添加（示例，使用 Git 仓库）：

```
ducia = { git = "https://github.com/USERNAME/REPO", branch = "main" }
```

然后在代码中调用：

```
let config_dir = std::path::PathBuf::from("./config");
let docs_dir = std::path::PathBuf::from("./docs");
tokio::spawn(async move {
    let _ = ducia::run_server("127.0.0.1:3001", config_dir, docs_dir).await;
});
```

发布为 crates.io

- 确保 `Cargo.toml` 中包含 `description`、`license`、`readme` 等字段。
- 使用 `cargo publish` 发布（需要 crates.io API token）。

许可证：MIT（仓库根目录的 `LICENSE` 文件）
