//! I18n API handlers
//!
//! GET /api/locales          — 返回可用语言列表
//! GET /api/locales/{locale} — 返回指定语言包完整内容

use crate::state::AppState;
use actix_web::{HttpResponse, Responder, web};
use std::path::PathBuf;

pub async fn list_locales(state: web::Data<AppState>) -> impl Responder {
    let locales = state.i18n.available_locales();
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": locales,
        "default": state.i18n.default_locale(),
    }))
}

pub async fn get_locale_pack(
    path: web::Path<String>,
    _state: web::Data<AppState>,
) -> impl Responder {
    let locale = path.into_inner();
    let i18n_dir = find_config_dir().join("i18n");
    let pack_path = i18n_dir.join(format!("{}.json", locale));

    match std::fs::read_to_string(&pack_path) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(pack) => HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": pack,
            })),
            Err(_) => HttpResponse::NotFound().json(serde_json::json!({
                "success": false, "message": "invalid locale pack"
            })),
        },
        Err(_) => HttpResponse::NotFound().json(serde_json::json!({
            "success": false, "message": "locale not found"
        })),
    }
}

fn find_config_dir() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut current = cwd.clone();
    loop {
        let cfg = current.join("config");
        if cfg.is_dir() {
            return cfg;
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return cwd.join("config"),
        }
    }
}
