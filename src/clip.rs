use fastembed::{ImageEmbedding};
use fastembed::{TextEmbedding};
use std::path::Path;

pub fn embed_query(model: &mut TextEmbedding, query: &str) -> anyhow::Result<Vec<f32>> {
    model.embed(vec![query], None)?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Failed to generate text query embedding"))
}

pub fn embed_image<P: AsRef<Path>>(model: &mut ImageEmbedding, image_path: P) -> anyhow::Result<Vec<f32>> {
    let path_str = image_path.as_ref().to_string_lossy().to_string();
    model.embed(vec![path_str], None)?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Failed to generate image embedding"))
}
