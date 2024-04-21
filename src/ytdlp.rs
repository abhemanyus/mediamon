const YT_DLP: &str = "yt-dlp";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("yt-dl {0}")]
    YTD(String),
    #[error("io")]
    IO(#[from] tokio::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn download_music(url: &str) -> Result<String> {
    let mut command = tokio::process::Command::new(YT_DLP);
    command.args([
        "--format-sort",
        "asr",
        "--format",
        "bestaudio",
        "-x",
        "--remux-video",
        "opus",
        "--embed-thumbnail",
        "--embed-metadata",
        "-o",
        "/tmp/%(title)s_%(id)s.%(ext)s",
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

pub async fn download_video(url: &str) -> Result<String> {
    let mut command = tokio::process::Command::new(YT_DLP);
    command.args([
        "--format-sort",
        "vbr,abr",
        "--format",
        "bv[vcodec!=h265]+ba",
        "--embed-thumbnail",
        "--embed-metadata",
        "-o",
        "/tmp/%(title)s_%(id)s.%(ext)s",
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
    download_music("https://www.youtube.com/watch?v=VFbhKZFzbzk")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_download_video() {
    download_video("https://www.youtube.com/watch?v=VFbhKZFzbzk")
        .await
        .unwrap();
}
