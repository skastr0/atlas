# context-map

**Deterministic knowledge base indexer for AI agents**

Generates multi-resolution markdown indexes of knowledge bases, solving the "AI doesn't know what it knows" problem through cheap, deterministic, static analysis.

## The Problem

AI agents working with knowledge bases (folders of markdown, PDFs, notes) face a chicken-and-egg problem: they need to know what exists before they can effectively search, but they can't search without knowing what to look for.

Existing solutions (embeddings, auto-memories, RAG) optimize for retrieval *given a query*, but don't solve the fundamental **discovery problem**.

## The Solution

`context-map` (alias: `cmap`) creates a **persistent, human-readable atlas** of your knowledge base:

- **ROOT_ATLAS.md** — Top-level map with folder signatures and key files
- **Per-folder INDEX.md** — Detailed listings with snippets and top terms
- **TERMS.md** — Concept-to-file mapping for topic navigation

The atlas is:
- **Deterministic** — No LLM, no randomness, fully reproducible
- **Incremental** — Only reprocesses changed files
- **Portable** — Plain files, works with any AI tool
- **Fast** — Sub-second for unchanged corpora

## Quick Start

```bash
# Build the binary
cargo build --release

# Initialize in your knowledge base
cd /path/to/your/kb
cmap init

# Build the index
cmap build

# View the atlas
cat .cmap/views/ROOT_ATLAS.md
```

## Commands

```
cmap init     # Initialize .cmap directory
cmap scan     # Scan for changes (fast fingerprint check)
cmap build    # Build/update index and generate views
cmap doctor   # Report issues (extraction failures, stale cache)
cmap clean    # Remove cached data
```

## What Gets Indexed

For each file, `cmap` extracts:

- **Title** — First heading or derived from filename
- **Snippet** — First paragraph (~400 chars)
- **Top terms** — TF-IDF weighted distinctive words
- **Top phrases** — Frequent bigrams and trigrams
- **Links** — Internal links, external URLs, citations (DOI, arXiv, ISBN)
- **Word/char count** — For quick size estimates

Across the corpus:

- **Global term frequencies** — Which terms are distinctive
- **Folder signatures** — Top terms/phrases per folder
- **Cross-references** — Term-to-file and phrase-to-file mappings

## Supported File Types

- Markdown (`.md`)
- Plain text (`.txt`)
- PDF (`.pdf`) — via `pdftotext` (Poppler)
- reStructuredText (`.rst`)
- Org mode (`.org`)

## Configuration

Edit `.cmap/config.toml`:

```toml
[scan]
ignore = [".git", ".cmap", "node_modules"]
include_extensions = ["md", "txt", "pdf"]

[extract]
max_file_size = 10000000  # 10MB
snippet_length = 400

[analyze]
top_terms = 20
top_phrases = 20
min_term_length = 3

[render]
atlas_folder_depth = 3
atlas_max_files_per_folder = 10
```

## PDF Support

PDF extraction requires `pdftotext` from Poppler:

```bash
# macOS
brew install poppler

# Ubuntu/Debian
apt install poppler-utils

# Fedora
dnf install poppler-utils
```

## Use with AI Agents

The generated markdown files are designed to be loaded as context:

1. **Always-on context**: Include `ROOT_ATLAS.md` in every conversation
2. **On-demand**: Load folder `INDEX.md` or `TERMS.md` when exploring specific topics
3. **Deep dive**: Reference specific file cards when needed

Example agent instruction:
```
You have access to the knowledge base atlas in .cmap/views/ROOT_ATLAS.md.
Use it to understand what exists before searching. When exploring a topic,
check the relevant folder INDEX.md for detailed file listings.
```

## Project Status

**Phase 1 (Complete):** Scaffold, CLI, configuration, types
**Phase 2 (In Progress):** Text extraction, analysis pipeline
**Phase 3 (Planned):** Global aggregation, view rendering
**Phase 4 (Planned):** Doctor diagnostics, polish

## Building from Source

```bash
# Development build
cargo build

# Release build (optimized, stripped)
cargo build --release

# Install locally
cargo install --path .
```

## License

MIT
