use crate::state::AppState;
use actix_web::{HttpResponse, Responder, web};

/// GET /api/cats/{id} — 获取单篇文档
pub async fn get_cat(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let storage = match state.plugins.storage() {
        Some(s) => s,
        None => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false
            }));
        }
    };

    match storage.get_doc(&id).await {
        Ok(Some(doc)) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": {
                "id": doc.id,
                "title": doc.title,
                "file": format!("{}.md", doc.id),
                "content": doc.content,
                "created_at": doc.created_at,
                "deprecated": doc.deprecated,
                "locked": doc.locked,
            }
        })),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "success": false, "message": "not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false, "message": e.to_string()
        })),
    }
}
