use fastembed::{ImageEmbedding, ImageInitOptions, ImageEmbeddingModel};
use fastembed::{TextEmbedding, TextInitOptions, EmbeddingModel};
use std::path::Path;

pub fn embed_query(query: &str) -> anyhow::Result<Vec<f32>> {
    let mut model = TextEmbedding::try_new(
        TextInitOptions::new(EmbeddingModel::ClipVitB32)
    )?;
    
    let embeddings = model.embed(vec![query], None)?;
    
    embeddings
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Failed to generate text query embedding"))
}

pub fn embed_image<P: AsRef<Path>>(image_path: P) -> anyhow::Result<Vec<f32>> {
    let mut model = ImageEmbedding::try_new(
        ImageInitOptions::new(ImageEmbeddingModel::ClipVitB32)
    )?;
    
    let path_str = image_path.as_ref().to_string_lossy().to_string();
    let embeddings = model.embed(vec![path_str], None)?;
    
    embeddings
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Failed to generate image embedding"))
}
