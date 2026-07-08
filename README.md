# VAGUE • [![License: AGPL v3](https://img.shields.io/badge/license-AGPL_3.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com) [![Issues Welcome](https://img.shields.io/badge/issues-welcome-brightgreen.svg)](https://github.com/ic0e/OS-Recon/issues) [![Maintenance](https://img.shields.io/badge/maintained-yes-brightgreen.svg)](https://github.com/ic0e/OS-Recon/graphs/commit-activity)

> A fully local semantic file search tool. Index a folder of documents and search them by meaning instead of exact keywords, powered by local embeddings through Ollama. Nothing ever leaves your machine.

[![Rust](https://img.shields.io/badge/Rust-2021-000000?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Ollama](https://img.shields.io/badge/Ollama-Local_Embeddings-000000?style=flat-square)](https://ollama.com/)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

Unlike filename or keyword search, vague understands what you're looking for. A query like "that one legal file" can correctly surface a tax document with zero literal word overlap, because search is done by comparing meaning, not text matching.

> !! Early MVP: text file indexing only, no CLI arguments yet, single-folder search.

## How it works

1. Files in a target folder are read and their text content is extracted.
2. Each file's text is converted into a vector embedding using a local Ollama model (`nomic-embed-text`), run entirely on your machine.
3. A search query is embedded the same way, then compared against every indexed file using cosine similarity.
4. Results are ranked by similarity and returned.

No API calls, no cloud services, no data sent anywhere. The only network activity is Ollama's one-time model download.

## Current Project Layout
```
vague/
├── src/
│   ├── main.rs        # entry point, wires indexing and search together
│   ├── embedder.rs    # calls local Ollama API to generate embeddings
│   ├── extract.rs     # reads text content from files
│   ├── indexer.rs      # walks a folder and builds the searchable index
│   └── store.rs        # cosine similarity and ranked search
```

## How to Run

Requires Rust and [Ollama](https://ollama.com/) installed locally.

**Pull the embedding model:**
```bash
ollama pull nomic-embed-text
```

**Clone and build:**
```bash
git clone https://github.com/ic0e/vague.git
cd vague
cargo build --release
```

**Usage:**
```bash
vague index 
vague search ""
```

`index` walks the given folder, embeds every file's contents, and saves the result to `vague_index.json` in the current directory. `search` loads that index and returns the top matches ranked by semantic similarity.

Example:
```bash
vague index testdata
vague search "that one legal file"
```

Ollama must be running in the background (it starts automatically after install) for both commands to work.

## Roadmap & Future Features
- PDF support
- CLI arguments for folder path and query
- Persisted index (avoid re embedding on every run)
- Image search support

## License
This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

> README.md last updated on July 7th 2026.
