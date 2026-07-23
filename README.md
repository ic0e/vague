# VAGUE • [![Rust](https://img.shields.io/badge/Rust-2024-000000?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/) [![CLIP](https://img.shields.io/badge/CLIP-Image_Embeddings-000000?style=flat-square)](https://github.com/openai/CLIP) [![fastembed](https://img.shields.io/badge/fastembed-ONNX_Runtime-000000?style=flat-square)](https://github.com/Anush008/fastembed-rs)

**vague** is a local semantic search CLI engine. Searches match meaning instead of just text. A query like "that one legal file" returns a tax document even with zero word overlap. A query like "screenshot of an error message" returns a .png, since images are embedded into a vector space with CLIP instead of being tagged with just metadata. Text and image results are ranked together in a single list.

> *!! Early MVP:* the program is still WIP, expect rough edges

## Search Features

**Text (.txt, .md, .pdf ...)**
1. File content is extracted and sent to a local text embedding model (nomic-embed-text) to produce a vector embedding.
2. A search query is embedded the same way and compared against every indexed text file using cosine similarity.
```bash
vague index folder
vague search search "cat" --limit 1
[1.0000] \\?\D:\folder\animals.txt
```
*use cases: finding specific documents and text files that have bad names, just by searching the context*

**Images (.png, .jpg, .jpeg, .webp)**
1. Images are embedded directly into vector space using **CLIP**, run locally via [fastembed](https://github.com/Anush008/fastembed-rs) (Rust bindings over ONNX Runtime).
2. Search queries are embedded through CLIP's own text encoder to land in the same space as the image vectors.
```bash
vague index folder
vague search "cat picture" --limit 1
[1.0000] \\?\D:\folder\cute_animals.png
```
*use cases: finding a specific picture from just a description, looking through random screenshots with no text*

**OCR text detection in images (.png, .jpg, .jpeg, .webp)**
1. The optional `--ocr` flag extracts readable text from images during indexing.
2. Extracted text is indexed alongside image vectors, so searches match words inside screenshots or diagrams.
*NOTE: Indexing with --ocr adds about ~0.4 seconds per image. Only use this if you need to search for text inside screenshots or diagrams.*
```bash
vague index folder --ocr
vague search "tower-http" --limit 1
[1.3000] \\?\D:\folder\screenshot-logs.png
```
*use cases: finding screenshots containing specific text, locating diagrams with labeled components*

## Installation

### Windows
1. Create a folder for vague like `C:\tools`
2. Download `vague.zip` from [Releases](https://github.com/ic0e/vague/releases) & extract its contents directly into `C:\tools` (ensure the `models` folder is in the same directory as `vague.exe`)
3. Add `C:\tools` to your PATH (if you want to use vague from any directory):
   - Open Settings -> search "environment variables" -> "Edit the system environment variables"
   - Click "Environment Variables" -> under "User variables" click "New"
   - Variable name: `PATH`
   - Variable value: `C:\tools` (or your chosen folder)
   - OK, restart your terminal
4. Run `vague --help` from any terminal

### Linux/macOS
1. Download `vague.zip` from [Releases](https://github.com/ic0e/vague/releases) and extract it.
2. Move the binary to your PATH and ensure the `models` folder is placed where `vague` expects it:
```bash
   sudo mv vague /usr/local/bin/
   sudo chmod +x /usr/local/bin/vague
   sudo mkdir -p /usr/local/share/vague
   sudo mv models/ /usr/local/share/vague/
```

*(Note: If `vague` looks for the `models` folder relative to the executable's path, you can alternatively keep the binary and the `models` folder together in a dedicated directory like `/opt/vague/` and symlink just the binary to `/usr/local/bin/vague`)*.

### From Source (requires Rust + C++ dev tools 2022)
```bash
git clone https://github.com/ic0e/vague
cd vague
cargo install --path .
```

### Updating
Download the latest `vague.exe` from [Releases](https://github.com/ic0e/vague/releases) and replace the old one in your PATH folder (e.g., `C:\tools`).

### Uninstall
Delete `vague.exe` from your PATH folder (e.g., `C:\tools`) and remove that folder from your PATH environment variable.

## Usage

### Setup & Basics

These are the first commands to run after installing vague.

```bash
vague --version   # prints the current version of vague
vague --help      # prints the current commands and their subcommands
vague setup       # downloads the text and image embedding models used for indexing
```

Running vague for the first time creates a settings file. Current settings:

* limit - how many results search returns

To change a setting:

```bash
vague settings <setting> <value>
```

Example:

```bash
vague settings limit 10   # search now returns 10 results instead of the default 5
```

### Indexing

Files need to be indexed before they can be searched.

```bash
vague index /path/to/your/files
```

This generates embeddings and creates a searchable index. Models download automatically to `~/.vague_cache` on first run (a few hundred MB, one time only).

Examples:

```bash
vague index .                       # indexes the cwd
cd pictures
vague index screenshots             # indexes a folder called screenshots
vague index screenshots/important   # indexes only that subfolder, not the rest of screenshots
```

Indexing can also run with OCR enabled, which extracts and saves text from images so it becomes searchable. This is slower than a normal index, so it's best used only on folders where the image text actually matters.

```bash
vague index <folder> --ocr
```

If you index without OCR first and want to add it later, you can run OCR separately afterward:

```bash
vague index <folder>
vague index <folder> --ocr   # only runs OCR on unindexed images, doesn't re-index existing files
```

Re-running index on a folder only picks up new files, it won't re-index files that are already in the index:

```bash
vague index .                 # indexes current folder
echo "text" > filename.txt    # adds a new file to the folder
vague index .                 # only indexes filename.txt, everything else isn't re-indexed
```

If existing files have changed and are no longer showing up correctly in search, the index needs to be overwritten instead. There are two ways to do this:

```bash
vague clear             # clears the entire index
vague index <folder>    # re-indexes from scratch
```

or in one step:

```bash
vague index . --overwrite   # clears and re-indexes in place, prompts with y/N
```

### Searching

```bash
vague search "find pictures of cats"
vague search "todo lists"
```

Results are ranked by relevance, both text and image matches in one output.

To see more or fewer results than the default without changing the setting permanently:

```bash
vague search "query" --limit 20   # returns 20 results for this search only
```


## Requirements for development

- [Rust](https://www.rust-lang.org/) (2024 edition)
- **A C++ build toolchain** — `fastembed`'s ONNX Runtime bindings need to compile/link against C++ tooling.
  - **Windows**: install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with the **"Desktop development with C++ 2022"** workload selected. (make sure version is 2022)
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`, etc.)
  - **Linux**: install build-essential (depends on distro, `sudo apt install build-essential`, etc.)

## Development

To build and run from source:
```bash
git clone https://github.com/ic0e/vague.git
cd vague
cargo build --release
cargo run -- index <folder>
cargo run -- search "<query>"
```

### OCR Support (for development)
OCR requires the detection and recognition models. To test OCR locally:

1. Place `text-detection.rten` and `text-recognition.rten` in `target/release/models/`
2. Run with `--release` only: `cargo run --release -- index <folder> --ocr`

OCR only works in **release mode**, debug builds makes it extremely slow due to [ocrs (docs)](https://docs.rs/crate/ocrs/latest).

## Project Layout For Devs
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

## Roadmap & Future Features
- Add showcase gifs in README
- Docx support
- Video support (frame extraction + CLIP embedding per frame)
- Further optimization of indexing and searching

## Contributing
Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on opening issues, testing features, and submitting pull requests.

## License
This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
