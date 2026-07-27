use actix_web::{HttpResponse, Responder, web};
use std::sync::Mutex;
use crate::db;
use crate::config;
use crate::AppState;

/// Handler that returns a list of available documents.
///
/// The response JSON includes a `success` flag, site name and an array of
/// documents (id, title, created_at) excluding deleted or deprecated items.
pub async fn list_cats(state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let docs_map = db::load_docs_map(&state.config_dir);
    let site_name = config::load_site_name(&state.config_dir);
    
    let docs: Vec<serde_json::Value> = docs_map.docs
        .iter()
        .filter(|(_, doc)| !doc.deleted && !doc.deprecated)
        .map(|(id, doc)| {
            serde_json::json!({
                "id": id,
                "title": doc.title,
                "created_at": doc.created_at,
            })
        })
        .collect();
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": docs,
        "siteName": site_name,
    }))
}
