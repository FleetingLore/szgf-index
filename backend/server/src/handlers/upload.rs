use crate::state::AppState;
use actix_web::{HttpResponse, Responder, web};
use ducia_core::doc::model::CreateDocRequest;

/// POST /api/cats — 创建文档
pub async fn upload_cat(
    req: web::Json<CreateDocRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let storage = match state.plugins.storage() {
        Some(s) => s,
        None => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false
            }));
        }
    };

    match storage.create_doc(req.into_inner()).await {
        Ok(meta) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "data": {
                "id": meta.id,
                "title": meta.title,
                "created_at": meta.created_at,
            }
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false, "message": e.to_string()
        })),
    }
}
