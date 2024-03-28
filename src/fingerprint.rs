use image::DynamicImage;
use image_hasher::{HashAlg, Hasher, HasherConfig};
pub struct Fingerprint {
    hasher: Hasher,
}

impl Default for Fingerprint {
    fn default() -> Self {
        Self::new()
    }
}

impl Fingerprint {
    pub fn new() -> Self {
        Self {
            hasher: HasherConfig::new()
                .hash_alg(HashAlg::Gradient)
                .hash_size(8, 8)
                .to_hasher(),
        }
    }
    pub fn fingerprint(&self, image: &DynamicImage) -> u64 {
        let hash = self.hasher.hash_image(image);
        let hash = hash.as_bytes();
        let mut hash_num = [0u8; 8];
        hash_num.iter_mut().zip(hash).for_each(|(a, b)| *a = *b);
        u64::from_be_bytes(hash_num)
    }
}
