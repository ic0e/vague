use walkdir::WalkDir;

pub struct IndexEntry {
    pub path: String,
    pub text: String,
    pub vector: Vec<f32>,
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

        let text = crate::extract::extract_text(&path)?;
        let vector = crate::embedder::embed_text(&text)?;

        let indexed_entry = IndexEntry{
            path: path.to_string_lossy().to_string(),
            text,
            vector,
        };
        
        result.push(indexed_entry);
    }

    Ok(result)
}
