#[tokio::main]
async fn main() {
    mediamon::ytdlp::download_music("https://www.youtube.com/watch?v=Ia0vVQnNGcc", "./music").await.unwrap();
}
