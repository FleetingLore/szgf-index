use crate::state::AppState;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use ducia_core::perm::model::Identity;
use std::collections::HashMap;

/// GET /api/admin/sequence
pub async fn get_sequence(_state: web::Data<AppState>) -> impl Responder {
    let config_dir = find_config_dir();

    let path = config_dir.join("sequence.json");
    let mut seq: Vec<usize> = Vec::new();

    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(arr) = json.get("sequence").and_then(|s| s.as_array()) {
                seq = arr
                    .iter()
                    .filter_map(|v| v.as_u64())
                    .map(|v| v as usize)
                    .collect();
            }
        }
    }

    HttpResponse::Ok().json(serde_json::json!({ "sequence": seq }))
}

/// POST /api/admin/session — 创建管理员会话
pub async fn create_session(state: web::Data<AppState>) -> impl Responder {
    let auth = match state.plugins.auth() {
        Some(a) => a.clone(),
        None => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false, "message": "no auth plugin"
            }));
        }
    };

    // 创建管理员身份
    let identity = Identity {
        id: "admin".into(),
        roles: vec!["admin".into()],
        permissions: vec!["*".into()],
        metadata: HashMap::new(),
    };

    match auth.create_session(&identity).await {
        Ok(token) => HttpResponse::Ok().json(serde_json::json!({ "token": token })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false, "message": e.to_string()
        })),
    }
}

/// GET /api/admin/session — 检查会话
pub async fn check_session(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let token = req
        .headers()
        .get("X-Session-Token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let auth = match state.plugins.auth() {
        Some(a) => a.clone(),
        None => return HttpResponse::Ok().json(serde_json::json!({ "isAdmin": false })),
    };

    let identity = auth.check_session(token).await;
    HttpResponse::Ok().json(serde_json::json!({
        "isAdmin": identity.is_some()
    }))
}

/// DELETE /api/admin/session — 销毁会话
pub async fn destroy_session(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let token = req
        .headers()
        .get("X-Session-Token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let auth = match state.plugins.auth() {
        Some(a) => a.clone(),
        None => return HttpResponse::Ok().json(serde_json::json!({ "success": true })),
    };

    let _ = auth.destroy_session(token).await;
    HttpResponse::Ok().json(serde_json::json!({ "success": true }))
}

fn find_config_dir() -> std::path::PathBuf {
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
