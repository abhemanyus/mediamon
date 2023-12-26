use mediamon::api::router;

#[tokio::main]
async fn main() {
    let router = router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, router).await.unwrap();
}
