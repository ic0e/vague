use std::path::Path;
use std::collections::HashSet;
use clap::{Parser, Subcommand};
use colored::*;
use walkdir::WalkDir;

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
    /// 
    /// By default this is additive: existing indexed folders are preserved,
    /// deleted files are pruned, and only new files are embedded.
    /// Use --overwrite to replace the entire index with this folder only.
    #[command(arg_required_else_help = true)]
    Index {
        /// The path to the target folder you want to index
        folder: String,

        /// Discard the entire existing index and replace it with this folder only.
        /// You will be asked to confirm before anything is deleted.
        #[arg(long)]
        overwrite: bool,
    },

    /// Searches your indexed files using a natural language query
    #[command(arg_required_else_help = true)]
    Search {
        /// The query string to look for (e.g., "legal documents")
        query: String,

        /// Optional: Number of results to return (default is 5)
        /// use --limit <num> or -l <num> 
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
    },

    /// Clears the entire search index. All indexed folders will need to be re-indexed.
    Clear,

    /// Downloads and caches both AI models (CLIP image + CLIP text) into ~/.vague_cache.
    /// Run this once after installation so index and search start instantly.
    Setup,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

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
        Commands::Clear => {
            if !db_path.exists() {
                println!("{}", "Index is already empty.".dimmed());
                return Ok(());
            }
            std::fs::remove_file(&db_path)?;
            println!("{}", "Index cleared.".green().bold());
        }

        Commands::Index { folder, overwrite } => {
            if overwrite {
                // warn and ask for confirmation before nuking the index
                println!("{}", "WARNING: --overwrite will discard the entire existing index and replace it with this folder only.".yellow().bold());
                print!("Continue? [y/N] ");
                use std::io::Write;
                std::io::stdout().flush()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
                    println!("{}", "Aborted.".dimmed());
                    return Ok(());
                }

                let entries = indexer::build_index(Path::new(&folder))?;
                store::save_index(&entries, db_path.to_str().unwrap())?;
                println!("Indexed {} successfully (overwrote previous index).", folder);
            } else {
                // SMART INDEXING

                // load whatever is already indexed (empty vec if nothing yet)
                let existing: Vec<indexer::IndexEntry> = if db_path.exists() {
                    store::load_index(db_path.to_str().unwrap())?
                } else {
                    vec![]
                };

                // canonicalize the target folder so path comparisons are reliable
                let canon_folder = match std::fs::canonicalize(&folder) {
                    Ok(p) => p,
                    Err(_) => {
                        eprintln!("{}", format!("Error: folder '{}' not found.", folder).red().bold());
                        return Ok(());
                    }
                };

                // walk the folder and collect all current file paths (canonicalized)
                let current_paths: Vec<std::path::PathBuf> = WalkDir::new(&folder)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .filter_map(|e| std::fs::canonicalize(e.path()).ok())
                    .collect();

                let _current_path_set: HashSet<String> = current_paths
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();

                // split existing entries into:
                //    - entries belonging to OTHER folders (always kept)
                //    - entries belonging to THIS folder (pruned if file deleted)
                let (in_folder, outside_folder): (Vec<_>, Vec<_>) =
                    existing.into_iter().partition(|e| {
                        if let Ok(canon_entry) = std::fs::canonicalize(&e.path) {
                            // file still exists — reliable canonical comparison
                            canon_entry.starts_with(&canon_folder)
                        } else {
                            // file is gone/moved — stored paths are already canonicalized
                            // absolute paths, so compare directly against canon_folder
                            Path::new(&e.path).starts_with(&canon_folder)
                        }
                    });

                // keep in-folder entries whose files still exist on disk
                let in_folder_count = in_folder.len();
                let kept: Vec<_> = in_folder
                    .into_iter()
                    .filter(|e| Path::new(&e.path).exists())
                    .collect();
                let pruned = in_folder_count - kept.len();
                let mut skipped: usize = 0;

                // build a set of already-indexed canonical paths so we skip them
                let already_indexed: HashSet<String> = kept
                    .iter()
                    .filter_map(|e| {
                        std::fs::canonicalize(&e.path)
                            .ok()
                            .map(|p| p.to_string_lossy().to_string())
                    })
                    .collect();

                // only embed files that aren't already in the index
                let new_paths: Vec<std::path::PathBuf> = current_paths
                    .into_iter()
                    .filter(|p| !already_indexed.contains(&p.to_string_lossy().to_string()))
                    .collect();

                let added = new_paths.len();

                if new_paths.is_empty() {
                    println!("{}", "Nothing new to index — all files already up to date.".dimmed());
                } else {
                    println!(
                        "{}",
                        format!("Indexing {} new file(s)...", added).cyan()
                    );
                }

                let new_entries = if new_paths.is_empty() {
                    vec![]
                } else {
                    indexer::index_file_list(&new_paths, &mut skipped)?
                };

                // merge: other folders + surviving in-folder entries + newly embedded
                let mut final_entries = outside_folder;
                final_entries.extend(kept);
                final_entries.extend(new_entries);

                store::save_index(&final_entries, db_path.to_str().unwrap())?;

                println!(
                    "Done. Index now has {} entries total  (+{} added, {} pruned, {} skipped).",
                    final_entries.len(),
                    added,
                    pruned,
                    skipped
                );
            }
        }

        Commands::Search { query, limit } => {
            let spinner = indicatif::ProgressBar::new_spinner();
            let _ = spinner.set_style(
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
                fastembed::TextInitOptions::new(fastembed::EmbeddingModel::ClipVitB32).with_cache_dir(cache_dir.clone())
            )? ;

            let clip_query = clip::embed_query(&mut text_model, &query)?;

            let text_query_batch = embedder::embed_text_batch(&[query.clone()], cache_dir.clone())?;
            let text_query = text_query_batch
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("Ollama returned an empty embedding batch for the search query"))?;

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

        Commands::Setup => {
            println!("{}", "Setting up vague — downloading models into ~/.vague_cache ...".cyan().bold());
            println!("{}", "This only needs to run once.".dimmed());

            // download the CLIP image model
            println!("\n{}", "[1/3] Downloading CLIP image model (ClipVitB32)...".yellow());
            fastembed::ImageEmbedding::try_new(
                fastembed::ImageInitOptions::new(fastembed::ImageEmbeddingModel::ClipVitB32)
                    .with_cache_dir(cache_dir.clone())
            )?;
            println!("{}", "     ✓ Image model ready.".green());

            // download the CLIP text model
            println!("\n{}", "[2/3] Downloading CLIP text model (ClipVitB32)...".yellow());
            fastembed::TextEmbedding::try_new(
                fastembed::TextInitOptions::new(fastembed::EmbeddingModel::ClipVitB32)
                    .with_cache_dir(cache_dir.clone())
            )?;
            println!("{}", "     ✓ CLIP text model ready.".green());
            
            println!("\n{}", "[3/3] Downloading text embedding model (Nomic)...".yellow());
            fastembed::TextEmbedding::try_new(
                fastembed::TextInitOptions::new(fastembed::EmbeddingModel::NomicEmbedTextV15)
                    .with_cache_dir(cache_dir.clone())
            )?;
            println!("{}", "     ✓ Text model ready.".green());

            println!("\n{}", "Setup complete! Models cached at:".bold());
            println!("  {}", cache_dir.display().to_string().white());
            println!("{}", "\nYou can now run 'vague index <folder>' and 'vague search <query>' instantly.".dimmed());
        }
    }

    Ok(())
}
