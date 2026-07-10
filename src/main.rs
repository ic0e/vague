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

    /// Downloads and caches both AI models (CLIP image + CLIP text) into ~/.vague_cache.
    /// Run this once after installation so index and search start instantly.
    Setup,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = Client::new();
    
    // set up a unified, global database directory (e.g., C:\Users\User\.vague_cache\)
    let cache_dir = dirs::home_dir()
        .map(|p| p.join(".vague_cache"))
        .unwrap_or_else(|| std::path::PathBuf::from(".fastembed_cache"));
    
    // make sure the global dir exists, if it doesn't create it
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }
    
    // index file in global directory, so you can search from anywhere
    let db_path = cache_dir.join("vague_index.json");
    
    match cli.command {
        Commands::Setup => {
            println!("{}", "Setting up vague — downloading models into ~/.vague_cache ...".cyan().bold());
            println!("{}", "This only needs to run once.".dimmed());

            // download the CLIP image model
            println!("\n{}", "[1/2] Downloading CLIP image model (ClipVitB32)...".yellow());
            fastembed::ImageEmbedding::try_new(
                fastembed::ImageInitOptions::new(fastembed::ImageEmbeddingModel::ClipVitB32)
                    .with_cache_dir(cache_dir.clone())
            )?;
            println!("{}", "     ✓ Image model ready.".green());

            // download the CLIP text model
            println!("\n{}", "[2/2] Downloading CLIP text model (ClipVitB32)...".yellow());
            fastembed::TextEmbedding::try_new(
                fastembed::TextInitOptions::new(fastembed::EmbeddingModel::ClipVitB32)
                    .with_cache_dir(cache_dir.clone())
            )?;
            println!("{}", "     ✓ Text model ready.".green());

            println!("\n{}", "Setup complete! Models cached at:".bold());
            println!("  {}", cache_dir.display().to_string().white());
            println!("{}", "\nYou can now run 'vague index <folder>' and 'vague search <query>' instantly.".dimmed());
        }
        Commands::Index { folder } => {
            let entries = indexer::build_index(Path::new(&folder))?;
            
            // save to global path instead of local directory
            store::save_index(&entries, db_path.to_str().unwrap())?;
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

            // check if database exists globally before loading 
            if !db_path.exists() {
                spinner.finish_and_clear();
                eprintln!("{}", "Error: No search index found. Run 'vague index <folder>' first!".red().bold());
                return Ok(());
            }

            let entries = store::load_index(db_path.to_str().unwrap())?;
                
            let mut text_model = fastembed::TextEmbedding::try_new(
                fastembed::TextInitOptions::new(fastembed::EmbeddingModel::ClipVitB32).with_cache_dir(cache_dir)
            )?;
            
            let clip_query = clip::embed_query(&mut text_model, &query)?;
            let text_query = embedder::embed_text(&client, &query)?;
            let results = store::search(&entries, &clip_query, &text_query, limit);

            spinner.finish_and_clear();
            
            println!("Top results for '{}':", query.bold().cyan());
            println!("{}", "─".repeat(50).dimmed());

            if results.is_empty() {
                println!("{}", "No results found.".red().italic());
            } else {
                for (entry, score) in results {
                    let score_color = if score > 0.8 {
                        format!("{:.4}", score).green().bold()
                    } else if score > 0.5 {
                        format!("{:.4}", score).yellow()
                    } else {
                        format!("{:.4}", score).dimmed()
                    };
                    
                    let display_path = if let Ok(abs_path) = std::path::Path::new(&entry.path).canonicalize() {
                        let mut url_path = abs_path.to_string_lossy().replace('\\', "/");
                        
                        if url_path.starts_with("//?/") {
                            url_path = url_path.replacen("//?/", "", 1);
                        }
                        
                        let file_url = format!("file:///{}", url_path);
                        format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", file_url, entry.path.white())
                    } else {
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
