//! 认证 API handlers
//!
//! POST /api/auth/register — 注册
//! POST /api/auth/login    — 登录
//! GET  /api/auth/me       — 当前用户信息

use actix_web::{HttpRequest, HttpResponse, Responder, web};
use ducia_auth_db::{AuthDb, LoginRequest, RegisterRequest};
use ducia_core::AuthPlugin;
use std::sync::Arc;

/// 注册
pub async fn register(
    req: web::Json<RegisterRequest>,
    auth: web::Data<Arc<AuthDb>>,
) -> impl Responder {
    match auth.register(&req) {
        Ok(user) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": user,
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": e.to_string(),
        })),
    }
}

/// 登录
pub async fn login(req: web::Json<LoginRequest>, auth: web::Data<Arc<AuthDb>>) -> impl Responder {
    match auth.login(&req) {
        Ok(resp) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": resp,
        })),
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "message": e.to_string(),
        })),
    }
}

/// 获取当前用户信息
pub async fn me(req: HttpRequest, auth: web::Data<Arc<AuthDb>>) -> impl Responder {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    match auth.check_session(token).await {
        Some(identity) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": {
                "id": identity.id,
                "roles": identity.roles,
                "username": identity.metadata.get("username"),
            }
        })),
        None => HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "message": "unauthorized"
        })),
    }
}
