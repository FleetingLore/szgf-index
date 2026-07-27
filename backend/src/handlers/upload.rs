use actix_web::{HttpResponse, Responder, web};
use std::sync::Mutex;
use crate::models::CreateDocReq;
use crate::db;
use crate::AppState;

/// Handler to create a new document from a JSON payload.
///
/// Persists the document content and updates the metadata map. Returns the
/// assigned document id and basic metadata on success.
pub async fn upload_cat(req: web::Json<CreateDocReq>, state: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = state.lock().unwrap();
    let mut docs_map = db::load_docs_map(&state.config_dir);
    let new_id = docs_map.next_id;
    let filename = format!("{}.md", new_id);
    
    let _ = db::save_doc_content(&state.docs_dir, &filename, &req.content);
    
    docs_map.docs.insert(new_id.to_string(), crate::models::DocMeta {
        title: req.title.clone(),
        file: filename,
        created_at: db::now_ms(),
        deprecated: false,
        deleted: false,
    });
    docs_map.next_id = new_id + 1;
    let _ = db::save_docs_map(&state.config_dir, &docs_map);
    
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "id": new_id.to_string(),
            "title": req.title,
            "created_at": db::now_ms(),
        }
    }))
}
