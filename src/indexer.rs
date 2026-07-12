use walkdir::WalkDir;
use serde::{Serialize, Deserialize};
use rayon::prelude::*;
use fastembed::{ImageEmbedding, ImageInitOptions, ImageEmbeddingModel};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::path::{Path, PathBuf};
use pdf_oxide::PdfDocument;
use std::error::Error;
use std::sync::Arc;

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
pub fn index_file_list(paths: &[PathBuf], skipped: &mut usize) -> anyhow::Result<Vec<IndexEntry>> {
    let cache_dir = dirs::home_dir()
        .map(|p| p.join(".vague_cache"))
        .unwrap_or_else(|| PathBuf::from(".fastembed_cache"));

    // Thread local macro, in order for the program to actually work with
    // multiple cores, the model can't be in a single variable to avoid it getting locked.
    thread_local! {
        static TLS_IMAGE_MODEL: std::cell::RefCell<Option<ImageEmbedding>> = std::cell::RefCell::new(None);
    }

    let skipped_atomic = Arc::new(AtomicUsize::new(0));

    let pb = indicatif::ProgressBar::new(paths.len() as u64);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} | {msg}")
            .expect("static template should compile")
            .progress_chars("#>-")
    );

    let result: Vec<IndexEntry> = paths
        .par_chunks(32) // process 32 files at a time per thread to optimize network batching
        .flat_map(|chunk| {
            let skipped = skipped_atomic.clone();
            let mut chunk_entries = Vec::new();
            let mut pending_texts = Vec::new();
            let mut pending_paths = Vec::new();

            for path in chunk {
                let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                pb.set_message(filename.to_string());

                match extension.as_str() {
                    "png" | "jpg" | "jpeg" | "webp" | "bmp" => {
                        // call the thread block and save the result to vector_res 
                        let vector_res = TLS_IMAGE_MODEL.with(|cell| {
                            let mut borrow = cell.borrow_mut();

                            if borrow.is_none() {
                                if let Ok(model) = ImageEmbedding::try_new(
                                    ImageInitOptions::new(ImageEmbeddingModel::ClipVitB32).with_cache_dir(cache_dir.clone())
                                ) {
                                    *borrow = Some(model);
                                }
                            }
                            if let Some(model) = borrow.as_mut() {
                                crate::clip::embed_image(model, path).ok()
                            } else {
                                None
                            }
                        });
                        if let Some(vector) = vector_res {
                                chunk_entries.push(IndexEntry {
                                path: path.to_string_lossy().to_string(),
                                text: filename.to_string(),
                                vector,
                                embedding_type: EmbeddingType::Clip,
                            });
                        }
                        pb.inc(1);
                    }
                    "txt" | "md"  => {
                        // extract text, but hold off on the HTTP call to batch it later
                        if let Some(text) = crate::extract::extract_text(path).ok() {
                            pending_texts.push(text);
                            pending_paths.push(path);
                        } else {
                            // if extraction fails, still increment progress bar
                            skipped.fetch_add(1, Ordering::Relaxed);
                            pb.inc(1);
                        }
                    }
                    "pdf" => {
                        if let Ok(text) = extract_pdf_text(path){
                            pending_texts.push(text);
                            pending_paths.push(path);
                        } else {
                            pb.inc(1);
                        }
                    }
                    _ => {
                        pb.inc(1); // skip unknown files, don't try to do anything with them
                    }
                }
            }

            // send all text files collected from this chunk to ollama in one batch request
            if !pending_texts.is_empty() {
                if let Some(vectors) = crate::embedder::embed_text_batch(&pending_texts, cache_dir.clone()).ok() {
                    // match the returned vectors back up to their corresponding files
                    for (i, vector) in vectors.into_iter().enumerate() {
                        let path = pending_paths[i];
                        let text = std::mem::take(&mut pending_texts[i]);
                        
                        chunk_entries.push(IndexEntry {
                            path: path.to_string_lossy().to_string(),
                            text,
                            vector,
                            embedding_type: EmbeddingType::Text,
                        });
                        pb.inc(1);
                    }
                } else {
                    // if the batch network call completely failed, make sure progress bar still updates
                    pb.inc(pending_paths.len() as u64);
                }
            }

            chunk_entries
        })
        .collect();

    pb.finish_with_message("Done!");

    *skipped = skipped_atomic.load(Ordering::Relaxed);
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
    
    let mut skipped: usize = 0;

    index_file_list(&paths, &mut skipped)
}

fn extract_pdf_text(path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let doc = PdfDocument::open(path)?;
    let mut text = String::new();

    for page_index in 0..doc.page_count().unwrap() {
            text.push_str(&doc.extract_text(page_index)?);
            text.push('\n'); // separate pages
    }
    
    Ok(text)
}
