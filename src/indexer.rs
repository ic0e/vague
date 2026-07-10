use walkdir::WalkDir;
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;
use rayon::prelude::*;
use fastembed::{ImageEmbedding, ImageInitOptions, ImageEmbeddingModel};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum EmbeddingType {
    Clip,
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
    
    // wrap the model in mutex
    let image_model = Mutex::new(ImageEmbedding::try_new(
        ImageInitOptions::new(ImageEmbeddingModel::ClipVitB32)
    )?);

    let entries: Vec<_> = WalkDir::new(folder)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

    let total_files = entries.len() as u64;
    let pb = indicatif::ProgressBar::new(total_files);

    pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} | {msg}")
                .expect("Static template should compile")
                .progress_chars("#>-")
        );
    
    let result: Vec<IndexEntry> = entries
            .par_iter() 
            .filter_map(|entry| {
                let path = entry.path();
                let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

                let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

                // update this per file
                pb.set_message(filename.to_string());
    
                let (text, vector, embedding_type) = match extension.as_str() {
                    "png" | "jpg" | "jpeg" | "webp" | "bmp" => {
                        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                        
                        // use mutex to get a mutable reference
                        let vector = {
                            let mut model_lock = image_model.lock().ok()?;
                            crate::clip::embed_image(&mut *model_lock, &path).ok()? 
                        };
                        
                        (filename, vector, EmbeddingType::Clip)
                    }
                    _ => {
                        let text = crate::extract::extract_text(&path).ok()?;
                        let vector = crate::embedder::embed_text(&client, &text).ok()?;
                        (text, vector, EmbeddingType::Text)
                    }
                };

                pb.inc(1); // tick the bar forward on each item from any Rayon thread
                
                Some(IndexEntry {
                    path: path.to_string_lossy().to_string(),
                    text,
                    vector,
                    embedding_type,
                })
            })
            .collect();

    pb.finish_with_message("Done!"); // finish the bar
    
    Ok(result)
}
