# VAGUE • [![License: AGPL v3](https://img.shields.io/badge/license-AGPL_3.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com) [![Issues Welcome](https://img.shields.io/badge/issues-welcome-brightgreen.svg)](https://github.com/ic0e/OS-Recon/issues) [![Maintenance](https://img.shields.io/badge/maintained-yes-brightgreen.svg)](https://github.com/ic0e/OS-Recon/graphs/commit-activity)

> A fully local, multimodal semantic search engine. Index a folder of documents *and* images, then search everything by meaning instead of exact keywords or filenames. Text is embedded with a local Ollama model; images are embedded with CLIP via `fastembed`. Nothing ever leaves your machine.

[![Rust](https://img.shields.io/badge/Rust-2021-000000?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Ollama](https://img.shields.io/badge/Ollama-Text_Embeddings-000000?style=flat-square)](https://ollama.com/)
[![CLIP](https://img.shields.io/badge/CLIP-Image_Embeddings-000000?style=flat-square)](https://github.com/openai/CLIP)
[![fastembed](https://img.shields.io/badge/fastembed-ONNX_Runtime-000000?style=flat-square)](https://github.com/Anush008/fastembed-rs)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

`vague` is a multimodal search pipeline. A query like *"that one legal file"* surfaces a tax document even with zero literal word overlap, because it compares meaning instead of matching text. A query like *"screenshot of an error message"* can surface a .png the same way, since images are embedded into an actual vector space with CLIP instead of just being tagged with metadata. Text and image results get ranked together in a single list, using normalized 0-1 scores so one modality doesn't drown out the other just because its raw similarity numbers happen to run higher.

> !! Early MVP: simple indexing, no CLI arguments for advanced filtering yet.

## How it works

**Text (`.txt`, `.md`, ...)**
1. File content is extracted and sent to a local Ollama model (`nomic-embed-text`) to produce a vector embedding.
2. A search query is embedded the same way and compared against every indexed text file using cosine similarity.

**Images (`.png`, `.jpg`, `.jpeg`, `.webp`)**
1. Images are embedded directly into vector space using **CLIP**, run locally via [`fastembed`](https://github.com/Anush008/fastembed-rs) (Rust bindings over ONNX Runtime). No captioning step, no LLM in the loop — the image itself becomes a vector.
2. Search queries are embedded through CLIP's own text encoder to land in the same space as the image vectors.

**Merged ranking**
Text and image results come from two different embedding spaces that aren't directly comparable on raw cosine score, so each result set is normalized to a 0-1 scale before being combined into a single ranked list — meaning a strong image match and a strong text match can both surface near the top, instead of one modality silently dominating the other because its raw scores happen to run higher.

No API calls, no cloud services, no data sent anywhere. The only network activity is the one-time model downloads (Ollama's `nomic-embed-text`, and CLIP's ONNX weights pulled automatically by `fastembed` on first run).

## Current Project Layout
```
vague/
├── src/
│   ├── main.rs        # entry point, wires indexing and search together
│   ├── embedder.rs    # calls local Ollama API for text embeddings
│   ├── clip.rs         # CLIP image + text-query embeddings via fastembed
│   ├── extract.rs     # reads text content from files
│   ├── indexer.rs      # walks a folder and builds the searchable index
│   └── store.rs        # normalized similarity scoring and merged ranked search
```

## Requirements

- [Rust](https://www.rust-lang.org/) (2021 edition)
- [Ollama](https://ollama.com/) installed and running locally, with the `nomic-embed-text` model pulled
- **A C++ build toolchain** — `fastembed`'s ONNX Runtime bindings need to compile/link against C++ tooling.
  - **Windows**: install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with the **"Desktop development with C++"** workload selected.
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: `build-essential` (or your distro's equivalent, e.g. `sudo apt install build-essential`)

## Installation

**From source:**
```bash
git clone https://github.com/ic0e/vague.git
cd vague
cargo install --path .
```

## Setup

Before using `vague`, pull the text embedding model and download the CLIP models:

```bash
ollama pull nomic-embed-text
vague setup
```

`vague setup` downloads CLIP's image and text models into `~/.vague_cache` on first run. This only needs to happen once - afterward, `index` and `search` start instantly. (NOTE: when running `vague search` or `vague index`, the program will automatically download the needed model.)

## Usage

Once set up, `vague` works as a global command from any directory. Ollama must be running in the background.

Basic syntax:
```bash
vague index <folder>
vague search "<query>"
vague search "<query>" --limit <num>
```

Examples:
```bash
vague index testdata
vague search "that one legal file"
vague search "a cute dog sitting on grass"
vague search "dog sitting on grass" --limit 6
```

For full command reference:
```bash
vague --help
vague index --help
vague search --help
```

## Development

To build and run from source:
```bash
git clone https://github.com/ic0e/vague.git
cd vague
cargo build --release
cargo run -- index <folder>
cargo run -- search "<query>"
```

## Roadmap & Future Features
- Docx support
- Video support (frame extraction + CLIP embedding per frame)
- Further optimization of indexing and searching

## License
This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

> README.md last updated on July 9th 2026.
