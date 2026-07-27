use actix_web::{HttpResponse, Responder, web, HttpRequest};
use std::sync::Mutex;
use std::collections::HashMap;
use uuid::Uuid;
use once_cell::sync::Lazy;
use crate::config;
use crate::AppState;

/// In-memory session store for simple admin authentication tokens.
///
/// This is a minimal, process-local session map used by the example admin
/// endpoints. It is not persistent and not suitable for production use.
static SESSIONS: Lazy<Mutex<HashMap<String, bool>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Return the admin sequence configuration.
pub async fn get_sequence(
    state: web::Data<Mutex<AppState>>,
) -> impl Responder {
    let state = state.lock().unwrap();
    let seq = config::load_admin_sequence(&state.config_dir);
    HttpResponse::Ok().json(serde_json::json!({ "sequence": seq }))
}

/// Create a new admin session token and return it to the caller.
pub async fn create_session() -> impl Responder {
    let token = Uuid::new_v4().to_string();
    SESSIONS.lock().unwrap().insert(token.clone(), true);
    HttpResponse::Ok().json(serde_json::json!({ "token": token }))
}

/// Check whether the provided `X-Session-Token` header represents a valid session.
pub async fn check_session(req: HttpRequest) -> impl Responder {
    let token = req.headers()
        .get("X-Session-Token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    let sessions = SESSIONS.lock().unwrap();
    let is_admin = sessions.contains_key(token);
    
    HttpResponse::Ok().json(serde_json::json!({ "isAdmin": is_admin }))
}

/// Destroy the session token provided via `X-Session-Token` header.
pub async fn destroy_session(req: HttpRequest) -> impl Responder {
    let token = req.headers()
        .get("X-Session-Token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    let mut sessions = SESSIONS.lock().unwrap();
    sessions.remove(token);
    
    HttpResponse::Ok().json(serde_json::json!({ "success": true }))
}
