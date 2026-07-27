use actix_web::{HttpResponse, Responder, web};
use std::sync::Mutex;
use crate::db;
use crate::AppState;

/// Handler to mark a document as deleted (soft delete).
///
/// Sets the `deleted` flag on the document metadata and persists the map.
pub async fn delete_cat(
    path: web::Path<String>,
    state: web::Data<Mutex<AppState>>,
) -> impl Responder {
    let id = path.into_inner();
    let state = state.lock().unwrap();
    
    let mut docs_map = db::load_docs_map(&state.config_dir);
    let doc = match docs_map.docs.get_mut(&id) {
        Some(d) => d,
        None => return HttpResponse::NotFound().json(serde_json::json!({"success": false})),
    };
    
    doc.deleted = true;
    let _ = db::save_docs_map(&state.config_dir, &docs_map);
    
    HttpResponse::Ok().json(serde_json::json!({"success": true}))
}
