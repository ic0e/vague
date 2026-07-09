use std::path::Path;
use clap::{Parser, Subcommand};
use reqwest::blocking::Client;

mod embedder;
mod extract;
mod indexer;
mod store;
mod clip;


#[derive(Parser)]
#[command(name = "vague")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Index {
        folder: String,
    },
    Search {
        query: String,
    },
}


fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let client = Client::new();
    
    match cli.command {
        Commands::Index { folder } => {
            let entries = indexer::build_index(Path::new(&folder))?;
            store::save_index(&entries, "vague_index.json")?;
            println!("Indexed {} successfully.", folder);
        }
        Commands::Search { query } => {
            let entries = store::load_index("vague_index.json")?;
            let clip_query = clip::embed_query(&query)?;
            let text_query = embedder::embed_text(&client, &query)?;
            let results = store::search(&entries, &clip_query, &text_query, 5);
            
            println!("Top results for '{}':", query);
                for (entry, score) in results {
                    println!("{:.4} - {}", score, entry.path);
                }
        }
    }

    Ok(())
}
