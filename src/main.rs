use std::path::Path;
use clap::{Parser, Subcommand};
use reqwest::blocking::Client;
use colored::*;

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
            let spinner = indicatif::ProgressBar::new_spinner();
                spinner.set_style(
                    indicatif::ProgressStyle::default_spinner()
                        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                        .template("{spinner:.green} {msg}")
                        .expect("Static template should compile")
                );
            spinner.set_message("Loading index and initializing models...");

            // spinner starts here
            let entries = store::load_index("vague_index.json")?;
            
            let mut text_model = fastembed::TextEmbedding::try_new(
                fastembed::TextInitOptions::new(fastembed::EmbeddingModel::ClipVitB32)
            )?;
            
            let clip_query = clip::embed_query(&mut text_model, &query)?;
            let text_query = embedder::embed_text(&client, &query)?;
            let results = store::search(&entries, &clip_query, &text_query, limit);

            // spinner stops here
            spinner.finish_and_clear();
            
            println!("Top results for '{}':", query.bold().cyan());
            println!("{}", "─".repeat(50).dimmed());

            if results.is_empty() {
                    println!("{}", "No results found.".red().italic());
                } else {
                    for (entry, score) in results {
                        // color coding based on confidence
                        let score_color = if score > 0.8 {
                            format!("{:.4}", score).green().bold()
                        } else if score > 0.5 {
                            format!("{:.4}", score).yellow()
                        } else {
                            format!("{:.4}", score).dimmed()
                        };
                        
                        let display_path = if let Ok(abs_path) = std::path::Path::new(&entry.path).canonicalize() {
                                let mut url_path = abs_path.to_string_lossy().replace('\\', "/");
                                
                                // a windows fix: canonicalize adds a UNC prefix "\\?\" which breaks file:// URLs
                                if url_path.starts_with("//?/") {
                                    url_path = url_path.replacen("//?/", "", 1);
                                }
                                
                                let file_url = format!("file:///{}", url_path);
                                
                                // OSC 8 escape sequence format: \x1b]8;;URL\x1b\TEXT\x1b]8;;\x1b\
                                format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", file_url, entry.path.white())
                            } else {
                                // fallback to regular text if path doesn't work
                                entry.path.white().to_string()
                            };
                        
                        println!("[{}] {}", score_color, display_path);
                    }
                }
            println!("{}", "─".repeat(50).dimmed());
        }
    }

    Ok(())
}
