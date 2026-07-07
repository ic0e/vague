use std::path::Path;

mod embedder;
mod extract;
mod indexer;

fn main() -> anyhow::Result<()> {
    let entries = indexer::build_index(Path::new("testdata"))?;

    println!("Indexed {} files", entries.len());
    for e in &entries {
        println!("- {} ({} chars, vector len {}", e.path, e.text.len(), e.vector.len());
    }

    Ok(())
}
