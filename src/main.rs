use log::info;
use mediamon::{api::router, database::Database, deepbooru::Jarvis};

#[tokio::main]
async fn main() {
    let jarvis = Jarvis::new("deepdanbooru.onnx").unwrap();
    let db = Database::new().await.unwrap();
    let router = router(jarvis, db);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("starting server...");
    axum::serve(listener, router).await.unwrap();
}
