const GALLERY_DL: &str = "gallery-dl";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("gallery-dl {0}")]
    GLD(String),
    #[error("io")]
    IO(#[from] tokio::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn download_art(url: &str) -> Result<String> {
    let mut command = tokio::process::Command::new(GALLERY_DL);
    command.arg(url);
    let output = match command.output().await {
        Ok(output) => output,
        Err(err) => return Err(Error::IO(err)),
    };
    if output.status.success() {
        Ok(String::from_utf8(output.stdout).unwrap_or("utf error".to_string()))
    } else {
        Err(Error::GLD(
            String::from_utf8(output.stderr).unwrap_or("utf error".to_string()),
        ))
    }
}

#[tokio::test]
async fn test_download_art() {
    download_art("https://cdn.discordapp.com/attachments/1030219956320731146/1230557127073071164/01504-generated-245199817002313.png?ex=6633c0a1&is=66214ba1&hm=4fd58b4bddede85791ce5d758b89fdec885bc6f3ca7f1a9253901e7b8fffa78a&")
        .await
        .unwrap();
}
