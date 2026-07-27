use crate::state::AppState;
use actix_web::{HttpResponse, Responder, web};

/// PUT /api/cats/{id}/deleted — 软删除
pub async fn delete_cat(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let storage = match state.plugins.storage() {
        Some(s) => s,
        None => {
            return HttpResponse::InternalServerError().json(serde_json::json!({"success": false}));
        }
    };

    match storage.update_meta(&id, None, Some(true), None).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false, "message": e.to_string()
        })),
    }
}
