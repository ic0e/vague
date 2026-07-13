# VAGUE • [![License: AGPL v3](https://img.shields.io/badge/license-AGPL_3.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com) [![Issues Welcome](https://img.shields.io/badge/issues-welcome-brightgreen.svg)](https://github.com/ic0e/vague/issues) [![Maintenance](https://img.shields.io/badge/maintained-yes-brightgreen.svg)](https://github.com/ic0e/vague/graphs/commit-activity)

> A fully local, multimodal semantic search engine CLI. Index a folder of documents *and* images, then search everything by meaning instead of exact keywords or filenames. Text is embedded with a local model, images are embedded with CLIP, both via `fastembed`. Nothing ever leaves your machine.

[![Rust](https://img.shields.io/badge/Rust-2024-000000?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![CLIP](https://img.shields.io/badge/CLIP-Image_Embeddings-000000?style=flat-square)](https://github.com/openai/CLIP)
[![fastembed](https://img.shields.io/badge/fastembed-ONNX_Runtime-000000?style=flat-square)](https://github.com/Anush008/fastembed-rs)

`vague` is a multimodal searching CLI. A query like *"that one legal file"* returns a tax document even with zero literal word overlap, because it compares meaning instead of matching text. A query like *"screenshot of an error message"* can return a .png the same way, since images are embedded into a vector space with CLIP instead of being tagged with metadata. Text and image results get ranked together in a single list, they are normalized with values from 0-1, since CLIP & nomic-text-embed don't normalize th e same way.

> !! Early MVP: the program is still WIP, expect rough edges

## How it works

**Text (`.txt`, `.md`, ...)**
1. File content is extracted and sent to a local text embedding model (`nomic-embed-text`) to produce a vector embedding.
2. A search query is embedded the same way and compared against every indexed text file using cosine similarity.

**Images (`.png`, `.jpg`, `.jpeg`, `.webp`)**
1. Images are embedded directly into vector space using **CLIP**, run locally via [`fastembed`](https://github.com/Anush008/fastembed-rs) (Rust bindings over ONNX Runtime).
2. Search queries are embedded through CLIP's own text encoder to land in the same space as the image vectors.

**Merged ranking**
Text and image results come from two different embedding spaces that aren't directly comparable on raw cosine score, so each result set is normalized to a 0-1 scale before being combined into a single list, meaning a strong image match and a strong text match both appear at the top (without one falling out).

## Current Project Layout
```
vague/
├── src/
│   ├── main.rs        # entry point, wires indexing and search together
│   ├── embedder.rs    # embeds text using `nomic-text-embed`
│   ├── clip.rs         # CLIP image + text-query embeddings via fastembed
│   ├── extract.rs     # reads text content from files
│   ├── indexer.rs      # walks a folder and builds the searchable index
│   └── store.rs        # normalized similarity scoring and merged ranked search
```

## Installation

### Windows
1. Download `vague.exe` from [Releases](https://github.com/ic0e/vague/releases)
2. Create a folder like `C:\tools` and put `vague.exe` there
3. Add `C:\tools` to your PATH (if you want to use this from any directory:)
   - Open Settings → search "environment variables" → "Edit the system environment variables"
   - Click "Environment Variables" → under "User variables" click "New"
   - Variable name: `PATH`
   - Variable value: `C:\tools`
   - Click OK, restart your terminal
4. Run `vague --help` from any terminal

### Linux/macOS
1. Download `vague` from [Releases](https://github.com/ic0e/vague/releases)
2. Move it to your PATH:
```bash
   sudo mv vague /usr/local/bin/
   chmod +x /usr/local/bin/vague
```
3. Run `vague --help` from anywhere

### From Source (requires Rust + C++ dev tools 2022)
```bash
git clone https://github.com/ic0e/vague
cd vague
cargo install --path .
```

### Updating
Download the latest `vague.exe` from [Releases](https://github.com/ic0e/vague/releases) and replace the old one in your PATH folder (e.g., `C:\tools`).

To check your current version: `vague --version`

### Uninstall
Delete `vague.exe` from your PATH folder (e.g., `C:\tools`) and remove that folder from your PATH environment variable.

## Requirements for development

- [Rust](https://www.rust-lang.org/) (2024 edition)
- **A C++ build toolchain** — `fastembed`'s ONNX Runtime bindings need to compile/link against C++ tooling.
  - **Windows**: install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with the **"Desktop development with C++"** workload selected.
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: `build-essential` (or your distro's equivalent, e.g. `sudo apt install build-essential`)

## Development

To build and run from source:
```bash
git clone https://github.com/ic0e/vague.git
cd vague
cargo build --release
cargo run -- index <folder>
cargo run -- search "<query>"
```

## Usage

**First time:** Index your files
```bash
vague index /path/to/your/files
```
This generates embeddings and creates a searchable index. Models download automatically to `~/.vague_cache` on first run (a few hundred MB, one time only).

**Then:** Search for text or images
```bash
vague search "find pictures of cats"
vague search "todo lists"
```

Results are ranked by relevance (both text and image matches in one output).

**Manage your index:**
```bash
vague clear              # Delete the current index
vague index /path --overwrite  # Re-index (you'll be prompted)
```

## Roadmap & Future Features
- Docx support
- Video support (frame extraction + CLIP embedding per frame)
- Further optimization of indexing and searching

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on opening issues, testing features, and submitting pull requests.

## License
This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
