use std::path::PathBuf;

use ulid::Ulid;

pub struct Storage {
    pub image: Image,
    pub video: Video,
}

pub struct Image {
    pub safe: Folder,
    pub r#unsafe: Folder,
}

pub struct Video {
    pub safe: Folder,
    pub r#unsafe: Folder,
}

pub struct Folder(PathBuf);

impl Folder {
    pub fn file_path(&self, ext: &str) -> PathBuf {
        self.0.join(Ulid::new().to_string()).with_extension(ext)
    }
}
