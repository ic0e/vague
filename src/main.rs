use std::path::Path;

mod embedder;
mod extract;
mod indexer;
mod store;

fn main() -> anyhow::Result<()> {
    let entries = indexer::build_index(Path::new("testdata"))?;

    let query = "that one legal file";
    let query_vector = embedder::embed_text(query)?;

    let results = store::search(&entries, &query_vector, 3);

    println!("Top results for '{}':", query);
    for (entry, score) in results {
        println!("{:.4} - {}", score, entry.path);
    }

    Ok(())
}
