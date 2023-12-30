use mediamon::{api::router, database::Database, deepbooru::Jarvis};

#[tokio::main]
async fn main() {
    let jarvis = Jarvis::new("deepdanbooru.onnx").unwrap();
    let db = Database::new().await.unwrap();
    let router = router(jarvis, db);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, router).await.unwrap();
}
