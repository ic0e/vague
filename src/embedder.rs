use fastembed::{TextEmbedding, TextInitOptions, EmbeddingModel};

thread_local! {
    static TLS_TEXT_MODEL: std::cell::RefCell<Option<TextEmbedding>> = std::cell::RefCell::new(None);
}

pub fn embed_text_batch(texts: &[String], cache_dir: std::path::PathBuf) -> anyhow::Result<Vec<Vec<f32>>> {
    TLS_TEXT_MODEL.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if borrow.is_none() {
            if let Ok(model) = TextEmbedding::try_new(
                TextInitOptions::new(EmbeddingModel::NomicEmbedTextV15)
                    .with_show_download_progress(true)
                    .with_cache_dir(cache_dir)
            ) {
                *borrow = Some(model);
            }
        }
        if let Some(model) = borrow.as_mut() {
            model.embed(texts, None)
        } else {
            Err(anyhow::anyhow!("Failed to load text embedding model"))
        }
    })
}
