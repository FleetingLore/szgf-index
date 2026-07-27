use crate::state::AppState;
use actix_web::{HttpResponse, Responder, web};
use ducia_core::doc::model::LockRequest;

/// PUT /api/cats/{id}/lock — 锁定/解锁文档
pub async fn set_lock(
    path: web::Path<String>,
    req: web::Json<LockRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner();
    let storage = match state.plugins.storage() {
        Some(s) => s,
        None => {
            return HttpResponse::InternalServerError().json(serde_json::json!({"success": false}));
        }
    };

    match storage.update_meta(&id, None, None, Some(req.locked)).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false, "message": e.to_string()
        })),
    }
}
