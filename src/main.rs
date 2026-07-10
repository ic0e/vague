use std::path::Path;
use clap::{Parser, Subcommand};
use reqwest::blocking::Client;

mod embedder;
mod extract;
mod indexer;
mod store;
mod clip;

/// Vague: A local semantic search engine
/// 
/// This tool uses CLIP and text embeddings to let you search through 
/// local directories using natural language queries.
#[derive(Parser)]
#[command(name = "vague")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Indexes a folder's contents for semantic search.
    #[command(arg_required_else_help = true)]
    Index {
        /// The path to the target folder you want to index
        folder: String,
    },
    
    /// Searches your indexed files using a natural language query
    #[command(arg_required_else_help = true)]
    Search {
        /// The query string to look for (e.g., "legal documents")
        query: String,

        /// Optional: Number of results to return (default is 5)
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
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
        Commands::Search { query, limit } => {
            let entries = store::load_index("vague_index.json")?;
            
            let mut text_model = fastembed::TextEmbedding::try_new(
                fastembed::TextInitOptions::new(fastembed::EmbeddingModel::ClipVitB32)
            )?;
            
            let clip_query = clip::embed_query(&mut text_model, &query)?;
            let text_query = embedder::embed_text(&client, &query)?;
            let results = store::search(&entries, &clip_query, &text_query, limit);
            
            println!("Top results for '{}':", query);
            for (entry, score) in results {
                println!("{:.4} - {}", score, entry.path);
            }
        }
    }

    Ok(())
}
