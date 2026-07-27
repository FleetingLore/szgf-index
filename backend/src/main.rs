//! Small binary to run the Ducia listing backend server locally.
use std::path::PathBuf;

/// Entrypoint for the backend binary; starts the HTTP server.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config_dir = PathBuf::from("../config");
    let docs_dir = PathBuf::from("../docs");
    ducia_listing_framework::run_server("127.0.0.1:3001", config_dir, docs_dir).await
}
