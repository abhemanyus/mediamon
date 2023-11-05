use std::{fs::read_to_string, path::Path};

use image::{imageops::FilterType, DynamicImage, ImageBuffer, Rgb};
use ndarray::{s, Array4, CowArray};
use ort::{
    tensor::OrtOwnedTensor, Environment, ExecutionProvider, GraphOptimizationLevel, Session,
    SessionBuilder, Value,
};

pub struct Jarvis {
    session: Session,
    tags: Vec<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ORT {0}")]
    Ort(#[from] ort::OrtError),
    #[error("IO {0}")]
    IO(#[from] std::io::Error),
    #[error("ANY {0}")]
    ANY(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Jarvis {
    pub fn new(model_path: impl AsRef<Path>, tags_path: impl AsRef<Path>) -> Result<Self> {
        let environment = Environment::builder()
            .with_name("deepbooru")
            .with_execution_providers([ExecutionProvider::CPU(Default::default())])
            .build()?
            .into_arc();
        let session = SessionBuilder::new(&environment)?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(std::thread::available_parallelism()?.get() as i16)?
            .with_model_from_file(&model_path)?;
        let tags = read_to_string(tags_path)?
            .split('\n')
            .map(|s| s.to_string())
            .collect();

        Ok(Self { tags, session })
    }

    pub fn infer_tags(&self, image: &DynamicImage, cutoff: Cutoff) -> Result<Vec<String>> {
        let resized = resize_padded(image, 512, 512);
        let resized_vec = resized.to_vec();
        let image = Array4::from_shape_vec((1, 512, 512, 3), resized_vec).or(Err(Error::ANY(
            "failed to resize array, impossible".to_string(),
        )))?;
        let mut image = image.mapv(|e| f32::from(e) / 255.0);
        image.swap_axes(2, 1);
        let image = CowArray::from(image).into_dyn();
        let inputs = vec![Value::from_array(self.session.allocator(), &image)?];
        let outputs: Vec<Value> = self.session.run(inputs)?;
        let generated_tags: OrtOwnedTensor<f32, _> = outputs[0].try_extract()?;
        let generated_tags = generated_tags.view();
        let best = &mut generated_tags
            .slice(s![0, ..])
            .iter()
            .cloned()
            .zip(0..)
            .collect::<Vec<(f32, usize)>>();
        best.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        Ok(best
            .iter()
            .filter(|(p, _)| *p >= cutoff.prob)
            .take(cutoff.count)
            .map(|(_, l)| self.tags[*l].clone())
            .collect())
    }
}

pub struct Cutoff {
    pub prob: f32,
    pub count: usize,
}

fn resize_padded(
    img: &DynamicImage,
    max_width: u32,
    max_height: u32,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut width = img.width();
    let mut height = img.height();
    let aspect_ratio = (width as f32) / (height as f32);

    if width > max_width || height < max_height {
        width = max_width;
        height = ((width as f32) / aspect_ratio) as u32;
    }

    if height > max_height || width < max_width {
        height = max_height;
        width = ((height as f32) * aspect_ratio) as u32;
    }

    let thumbnail = img.resize_exact(width, height, FilterType::Gaussian);
    let mut img = ImageBuffer::from_pixel(max_width, max_height, Rgb([255, 255, 255]));
    image::imageops::overlay(
        &mut img,
        &thumbnail.to_rgb8(),
        (max_width - width) as i64 / 2,
        (max_height - height) as i64 / 2,
    );
    img
}
