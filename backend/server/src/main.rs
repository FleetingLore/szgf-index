//! Ducia listing framework — server binary
//!
//! 组装插件、加载配置、启动 HTTP 服务。
//! 通过 config/settings.json 选择存储和认证后端。

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, middleware, web};
use ducia_auth_db::{AuthConfig, AuthDb};
use ducia_auth_simple::SimpleAuth;
use ducia_core::I18nManager;
use ducia_core::perm::model::RoleConfig;
use ducia_core::plugin::registry::PluginRegistry;
use ducia_server::handlers;
use ducia_server::state::AppState;
use ducia_storage_fs::FsStorage;
use std::path::PathBuf;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cwd = std::env::current_dir().unwrap_or_default();
    // 找到项目根目录（包含 config/ 的那一级）
    let base_dir = find_project_root(&cwd);

    let config_dir = base_dir.join("config");
    let docs_dir = base_dir.join("docs");
    let data_dir = base_dir.join("data");

    let _ = std::fs::create_dir_all(&config_dir);
    let _ = std::fs::create_dir_all(&docs_dir);
    let _ = std::fs::create_dir_all(&data_dir);
    let _ = std::fs::create_dir_all(&config_dir.join("i18n"));

    // 加载角色配置
    let role_config = load_role_config(&config_dir);

    // 加载 I18n
    let i18n = I18nManager::load(&config_dir, "zh-CN");

    // 根据 settings.json 选择存储后端
    let use_db = is_db_enabled(&config_dir);

    // 创建存储插件
    let storage_plugin = if use_db {
        let db_path = data_dir.join("ducia.db");
        println!("Using SQLite storage: {:?}", db_path);
        Box::new(
            ducia_storage_sqlite::SqliteStorage::new(db_path, docs_dir.clone())
                .expect("failed to open sqlite database"),
        ) as Box<dyn ducia_core::doc::repo::DocRepository>
    } else {
        println!("Using filesystem storage");
        Box::new(FsStorage::new(
            config_dir.clone(),
            data_dir.clone(),
            docs_dir.clone(),
        ))
    };

    // 创建认证插件（优先 auth-db，fallback auth-simple）
    let auth_plugin = if config_dir.join("auth.json").exists() {
        println!("Using auth-db (database authentication)");
        let auth_config: AuthConfig = serde_json::from_str(
            &std::fs::read_to_string(config_dir.join("auth.json")).unwrap_or_default(),
        )
        .unwrap_or(AuthConfig { jwt_secret: None });
        Arc::new(
            AuthDb::new(data_dir.join("auth.db"), auth_config)
                .expect("failed to open auth database"),
        ) as Arc<dyn ducia_core::plugin::auth::AuthPlugin>
    } else {
        println!("Using auth-simple (sequence code authentication)");
        Arc::new(SimpleAuth::new(config_dir.clone()))
    };

    let registry = PluginRegistry::new()
        .with_auth(auth_plugin.clone())
        .with_storage(storage_plugin);

    let state = web::Data::new(AppState::new(registry, role_config, i18n));

    // 调试模式
    if std::env::var("DUCIA_DEBUG").unwrap_or_default() == "true" {
        println!("Debug mode enabled (DUCIA_DEBUG=true)");
    }

    println!("Ducia server on http://127.0.0.1:3001");

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::new("%s %{METHOD} %{PATH}%{QUERY}"))
            .app_data(state.clone())
            .app_data(web::Data::new(auth_plugin.clone()))
            // 文档 API
            .route("/api/cats", web::get().to(handlers::list::list_cats))
            .route("/api/cats/{id}", web::get().to(handlers::get::get_cat))
            .route("/api/cats", web::post().to(handlers::upload::upload_cat))
            .route(
                "/api/cats/{id}/deprecated",
                web::put().to(handlers::deprecated::set_deprecated),
            )
            .route(
                "/api/cats/{id}/deleted",
                web::put().to(handlers::delete::delete_cat),
            )
            .route(
                "/api/cats/{id}/lock",
                web::put().to(handlers::lock::set_lock),
            )
            // 管理 API
            .route(
                "/api/admin/sequence",
                web::get().to(handlers::admin::get_sequence),
            )
            .route(
                "/api/admin/session",
                web::post().to(handlers::admin::create_session),
            )
            .route(
                "/api/admin/session",
                web::get().to(handlers::admin::check_session),
            )
            .route(
                "/api/admin/session",
                web::delete().to(handlers::admin::destroy_session),
            )
            // 认证 API
            .route(
                "/api/auth/register",
                web::post().to(handlers::auth_handler::register),
            )
            .route(
                "/api/auth/login",
                web::post().to(handlers::auth_handler::login),
            )
            .route("/api/auth/me", web::get().to(handlers::auth_handler::me))
            // I18n API
            .route("/api/locales", web::get().to(handlers::i18n::list_locales))
            .route(
                "/api/locales/{locale}",
                web::get().to(handlers::i18n::get_locale_pack),
            )
            // SPA 静态文件 + 路由 fallback：先尝试精确文件，失败返回 index.html
            .route("/{tail:.*}", web::get().to(spa_fallback))
    })
    .bind("0.0.0.0:3001")?
    .run()
    .await
}

/// SPA fallback：先尝试返回精确静态文件，失败则返回 index.html
async fn spa_fallback(req: actix_web::HttpRequest) -> impl Responder {
    let cwd = std::env::current_dir().unwrap_or_default();
    let base_dir = find_project_root(&cwd);
    let dist_dir = base_dir.join("dist");
    let path = req.path().trim_start_matches('/');
    let file_path = if path.is_empty() {
        dist_dir.join("index.html")
    } else {
        dist_dir.join(path)
    };

    // 尝试服务精确文件
    if file_path.is_file() {
        if let Ok(data) = tokio::fs::read(&file_path).await {
            let ct = actix_files::file_extension_to_mime(
                file_path.extension().and_then(|e| e.to_str()).unwrap_or(""),
            );
            return HttpResponse::Ok().content_type(ct).body(data);
        }
    }

    // SPA fallback：返回 index.html，调试模式下注入标记
    match tokio::fs::read_to_string(dist_dir.join("index.html")).await {
        Ok(html) => {
            let html = if std::env::var("DUCIA_DEBUG").unwrap_or_default() == "true" {
                html.replace("<head>", "<head><script>window.__DUCIA_DEBUG=true</script>")
            } else {
                html
            };
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(_) => HttpResponse::NotFound().body("Not Found"),
    }
}

/// 向上查找包含 config/ 目录的项目根
fn find_project_root(cwd: &PathBuf) -> PathBuf {
    let mut current = cwd.clone();
    loop {
        if current.join("config").is_dir() {
            return current;
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return cwd.clone(),
        }
    }
}

fn load_role_config(config_dir: &PathBuf) -> RoleConfig {
    let path = config_dir.join("roles.json");
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| default_role_config()),
        Err(_) => default_role_config(),
    }
}

fn is_db_enabled(config_dir: &PathBuf) -> bool {
    let path = config_dir.join("settings.json");
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            return json
                .get("use_database")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
        }
    }
    false
}

fn default_role_config() -> RoleConfig {
    let mut roles = std::collections::HashMap::new();
    roles.insert(
        "admin".into(),
        ducia_core::perm::model::RoleDef {
            permissions: vec!["*".into()],
            extends: vec![],
        },
    );
    roles.insert(
        "viewer".into(),
        ducia_core::perm::model::RoleDef {
            permissions: vec!["doc:read".into()],
            extends: vec![],
        },
    );
    RoleConfig { roles }
}
