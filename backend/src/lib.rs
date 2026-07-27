//! Backend framework for the Ducia listing service.
//!
//! This crate provides a lightweight HTTP server and shared application state
//! used by the frontend to manage and serve documents.
pub mod config;
pub mod db;
pub mod handlers;
pub mod models;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use std::path::PathBuf;
use std::sync::Mutex;

/// Shared application state accessible by request handlers.
///
/// Holds paths to configuration and document directories used at runtime.
pub struct AppState {
    /// Directory where configuration files such as `site.json` and `docs.json` reside.
    pub config_dir: PathBuf,
    /// Directory where document markdown files are stored.
    pub docs_dir: PathBuf,
}

/// Start and run the HTTP server for the application.
///
/// # Arguments
///
/// * `bind_addr` - The address and port to bind (for example `"127.0.0.1:3001"`).
/// * `config_dir` - Path to the directory containing configuration files.
/// * `docs_dir` - Path to the directory storing document content files.
pub async fn run_server(
    bind_addr: &str,
    config_dir: PathBuf,
    docs_dir: PathBuf,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&config_dir)?;
    std::fs::create_dir_all(&docs_dir)?;

    let state = web::Data::new(Mutex::new(AppState {
        config_dir,
        docs_dir,
    }));

    println!("Server on http://{}", bind_addr);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(state.clone())
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
    })
    .bind(bind_addr)?
    .run()
    .await
}
