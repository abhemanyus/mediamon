pub enum ImageRating {
    Safe,
    Questionable,
    Explicit,
}
pub async fn save_image(image: &[u8], rating: Option<ImageRating>) {}
