use actix_web::{HttpResponse, Responder, web};
use std::sync::Mutex;
use crate::models::DeprecatedReq;
use crate::db;
use crate::AppState;

/// Handler to mark a document as deprecated or to unmark it.
///
/// Expects a boolean `deprecated` field in the JSON payload. Returns `404`
/// if the document does not exist or has been deleted.
pub async fn set_deprecated(
    path: web::Path<String>,
    req: web::Json<DeprecatedReq>,
    state: web::Data<Mutex<AppState>>,
) -> impl Responder {
    let id = path.into_inner();
    let state = state.lock().unwrap();
    let mut docs_map = db::load_docs_map(&state.config_dir);
    
    let doc = match docs_map.docs.get_mut(&id) {
        Some(d) => d,
        None => return HttpResponse::NotFound().json(serde_json::json!({"success": false})),
    };
    
    if doc.deleted {
        return HttpResponse::NotFound().json(serde_json::json!({"success": false}));
    }
    
    doc.deprecated = req.deprecated;
    let _ = db::save_docs_map(&state.config_dir, &docs_map);
    
    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}
