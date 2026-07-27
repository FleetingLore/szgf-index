use crate::state::AppState;
use actix_web::{HttpResponse, Responder, web};
use ducia_core::doc::model::DeprecatedRequest;

/// PUT /api/cats/{id}/deprecated
pub async fn set_deprecated(
    path: web::Path<String>,
    req: web::Json<DeprecatedRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner();
    let storage = match state.plugins.storage() {
        Some(s) => s,
        None => {
            return HttpResponse::InternalServerError().json(serde_json::json!({"success": false}));
        }
    };

    match storage
        .update_meta(&id, Some(req.deprecated), None, None)
        .await
    {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false, "message": e.to_string()
        })),
    }
}
