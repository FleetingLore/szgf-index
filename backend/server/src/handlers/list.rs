use actix_web::{web, HttpResponse, Responder};
use crate::state::AppState;

/// GET /api/cats — 列出所有文档
pub async fn list_cats(state: web::Data<AppState>) -> impl Responder {
    let storage = match state.plugins.storage() {
        Some(s) => s,
        None => return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false, "message": "no storage plugin"
        })),
    };

    match storage.list_docs(false).await {
        Ok(docs) => {
            let site_name = storage.site_name().await;
            let items: Vec<serde_json::Value> = docs.iter().map(|d| {
                serde_json::json!({
                    "id": d.id,
                    "title": d.title,
                    "created_at": d.created_at,
                })
            }).collect();

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "data": items,
                "siteName": site_name,
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false, "message": e.to_string()
        })),
    }
}
