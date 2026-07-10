use walkdir::WalkDir;
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;
use rayon::prelude::*;
use fastembed::{ImageEmbedding, ImageInitOptions, ImageEmbeddingModel};
use std::sync::Mutex;
use std::path::{Path, PathBuf};

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

/// Embeds a specific list of file paths and returns their index entries.
/// Used for both full indexing and incremental (new-files-only) indexing.
pub fn index_file_list(paths: &[PathBuf]) -> anyhow::Result<Vec<IndexEntry>> {
    let client = Client::new();

    let cache_dir = dirs::home_dir()
        .map(|p| p.join(".vague_cache"))
        .unwrap_or_else(|| PathBuf::from(".fastembed_cache"));

    // force model to use global cache dir so it never downloads into the working directory
    let image_model = Mutex::new(ImageEmbedding::try_new(
        ImageInitOptions::new(ImageEmbeddingModel::ClipVitB32).with_cache_dir(cache_dir)
    )?);

    let pb = indicatif::ProgressBar::new(paths.len() as u64);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} | {msg}")
            .expect("static template should compile")
            .progress_chars("#>-")
    );

    let result: Vec<IndexEntry> = paths
        .par_iter()
        .filter_map(|path| {
            let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            pb.set_message(filename.to_string());

            let (text, vector, embedding_type) = match extension.as_str() {
                "png" | "jpg" | "jpeg" | "webp" | "bmp" => {
                    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    let vector = {
                        let mut model_lock = image_model.lock().ok()?;
                        crate::clip::embed_image(&mut *model_lock, path).ok()?
                    };
                    (filename, vector, EmbeddingType::Clip)
                }
                _ => {
                    let text = crate::extract::extract_text(path).ok()?;
                    let vector = crate::embedder::embed_text(&client, &text).ok()?;
                    (text, vector, EmbeddingType::Text)
                }
            };

            pb.inc(1);

            Some(IndexEntry {
                path: path.to_string_lossy().to_string(),
                text,
                vector,
                embedding_type,
            })
        })
        .collect();

    pb.finish_with_message("Done!");

    Ok(result)
}

/// Indexes all files under `folder` from scratch.
/// Used by `--overwrite` mode.
pub fn build_index(folder: &Path) -> anyhow::Result<Vec<IndexEntry>> {
    let paths: Vec<PathBuf> = WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    index_file_list(&paths)
}
