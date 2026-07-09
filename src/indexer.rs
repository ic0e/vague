use walkdir::WalkDir;
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;
use rayon::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum EmbeddingType {
    // CLIP processing
    Clip,
    // the normal text encoder
    Text,
}

#[derive(Serialize, Deserialize)]
pub struct IndexEntry {
    pub path: String,
    pub text: String,
    pub vector: Vec<f32>,
    pub embedding_type: EmbeddingType,
}

pub fn build_index(folder: &std::path::Path) -> anyhow::Result<Vec<IndexEntry>> {
    let client = Client::new();
    let entries: Vec<_> = WalkDir::new(folder)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();
    
    
    
    let result: Vec<IndexEntry> = entries
            .par_iter() // parallel
            .filter_map(|entry| {
                let path = entry.path();
                let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    
                let (text, vector, embedding_type) = match extension.as_str() {
                    "png" | "jpg" | "jpeg" | "webp" | "bmp" => {
                        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                        let vector = crate::clip::embed_image(&path).ok()?; 
                        (filename, vector, EmbeddingType::Clip)
                    }
                    _ => {
                        let text = crate::extract::extract_text(&path).ok()?;
                        let vector = crate::embedder::embed_text(&client, &text).ok()?;
                        (text, vector, EmbeddingType::Text)
                    }
                };
    
                Some(IndexEntry {
                    path: path.to_string_lossy().to_string(),
                    text,
                    vector,
                    embedding_type,
                })
            })
            .collect(); // get all the vectors back safely

    Ok(result)
}
