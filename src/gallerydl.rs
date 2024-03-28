const GALLERY_DL: &str = "gallery-dl";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("gallery-dl {0}")]
    YTD(String),
    #[error("io")]
    IO(#[from] tokio::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn download_art(url: &str, dir: &str) -> Result<String> {
    let mut command = tokio::process::Command::new(GALLERY_DL);
    command.args([
        "--format-sort",
        "asr",
        "--format",
        "bestaudio",
        "--recode-video",
        "opus",
        "--embed-thumbnail",
        "--embed-metadata",
        "-o",
        &format!("{}/%(title)s_%(id)s.%(ext)s", dir),
        "--print",
        "after_move:filepath",
        url,
    ]);
    let output = match command.output().await {
        Ok(output) => output,
        Err(err) => return Err(Error::IO(err)),
    };
    if output.status.success() {
        Ok(String::from_utf8(output.stdout).unwrap_or("utf error".to_string()))
    } else {
        Err(Error::YTD(
            String::from_utf8(output.stderr).unwrap_or("utf error".to_string()),
        ))
    }
}

#[tokio::test]
async fn test_download_music() {
    download_art("https://www.youtube.com/watch?v=VFbhKZFzbzk", "./music")
        .await
        .unwrap();
}
