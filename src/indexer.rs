use walkdir::WalkDir;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum EmbeddingType {
    /// CLIP image-encoder vector (for PNG/JPG/etc.)
    Clip,
    /// Nomic text-encoder vector (for txt/md/etc.)
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
    let mut result: Vec<IndexEntry> = Vec::new();
    
    for entry in WalkDir::new(folder) {
        let entry = match entry {
                Ok(e) => e,
                Err(err) => {
                    println!("Failed on: {:?}", err.path());
                    return Err(err.into());
                }
            };
        let path = entry.path();

        if !entry.file_type().is_file() {
            continue;
        }

        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

        let (text, vector, embedding_type) = match extension.as_str() {
            "png" | "jpg" | "jpeg" | "webp" | "bmp" => {
                let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                let vector = crate::clip::embed_image(&path)?;
                (filename, vector, EmbeddingType::Clip)
            }
            _ => {
                let text = crate::extract::extract_text(&path)?;
                let vector = crate::embedder::embed_text(&text)?;
                (text, vector, EmbeddingType::Text)
            }
        };

        let indexed_entry = IndexEntry{
            path: path.to_string_lossy().to_string(),
            text,
            vector,
            embedding_type,
        };
        
        result.push(indexed_entry);
    }

    Ok(result)
}
