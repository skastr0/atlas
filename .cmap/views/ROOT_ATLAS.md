# Knowledge Base Atlas

_Auto-generated map of this knowledge base. Use this to understand what exists before searching._

## Overview

- **Total files:** 40
- **Total words:** 1381
- **Folders:** 10

## Folder Structure

- **(root)/** (40 files) — knowledge base, folder index.md, text extraction
  - **src/** (39 files) — knowledge base, relative path, text extraction
    - **aggregate/** (3 files) — global term, global term index, term index
    - **analyze/** (8 files) — basic english, computation compute, compute idf
    - **cache/** (3 files) — cache management, cached features, fingerprint cache
    - **cli/** (6 files) — command report, cmap clean, command initialize
    - **config/** (1 files) — document frequency, number top, term length maximum
    - **extract/** (7 files) — text extraction, extract content, tree sitter
    - **render/** (5 files) — generation generate, path list, folder index.md
    - **scan/** (3 files) — fingerprints jsonl, change detection, ignore patterns

## Objective Slices

### Largest Files

- **context-map** (445) - README.md
- **Core data types for the indexer** (279) - src/types.rs
- **context-map: Deterministic knowledge base indexer for AI agents** (80) - src/main.rs
- **Configuration types and defaults** (79) - src/config/mod.rs
- **Text extraction from various file types** (39) - src/extract/mod.rs
- **Unicode-aware word tokenization** (37) - src/analyze/tokenize.rs
- **File fingerprinting for change detection** (31) - src/scan/fingerprint.rs
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** (30) - src/analyze/mod.rs
- **TF-IDF computation** (24) - src/analyze/tfidf.rs
- **Global term index and document frequency** (22) - src/aggregate/term_index.rs

### Most Connected

- **`cmap build` command - Build index and generate views** (13) - src/cli/build.rs
- **YAKE keyword extraction** (8) - src/analyze/yake.rs
- **ROOT_ATLAS.md generation** (4) - src/render/atlas.rs
- **Per-file feature computation** (4) - src/analyze/features.rs
- **Unicode-aware word tokenization** (4) - src/analyze/tokenize.rs
- **RAKE (Rapid Automatic Keyword Extraction)** (4) - src/analyze/rake.rs
- **Text extraction from various file types** (4) - src/extract/mod.rs
- **Per-folder INDEX.md generation** (3) - src/render/folder_index.rs
- **`cmap init` command - Initialize .cmap directory** (3) - src/cli/init.rs
- **PDF text extraction via pdftotext** (3) - src/extract/pdf.rs

### Most Exported Symbols

- **Core data types for the indexer** (15) - src/types.rs
- **Unicode-aware word tokenization** (7) - src/analyze/tokenize.rs
- **CONNECTIONS.md and graph generation** (7) - src/render/graph.rs
- **Configuration types and defaults** (6) - src/config/mod.rs
- **File fingerprinting for change detection** (5) - src/scan/fingerprint.rs
- **TF-IDF computation** (4) - src/analyze/tfidf.rs
- **Feature cache management** (4) - src/cache/features.rs
- **use crate::types::FileType;** (4) - src/extract/treesitter.rs
- **N-gram extraction (bigrams, trigrams)** (3) - src/analyze/ngrams.rs
- **RAKE (Rapid Automatic Keyword Extraction)** (3) - src/analyze/rake.rs

### Most Distinctive

- **TERMS.md generation** (1.436) - src/render/term_index.rs
- **CLI command implementations** (1.332) - src/cli/mod.rs
- **YAKE keyword extraction** (1.332) - src/analyze/yake.rs
- **ROOT_ATLAS.md generation** (1.321) - src/render/atlas.rs
- **Fingerprint cache management** (1.197) - src/cache/fingerprints.rs
- **`cmap scan` command - Scan files and update fingerprints** (1.197) - src/cli/scan.rs
- **Markdown text extraction** (1.026) - src/extract/markdown.rs
- **Plain text extraction** (1.026) - src/extract/plaintext.rs
- **PDF text extraction via pdftotext** (1.016) - src/extract/pdf.rs
- **`cmap build` command - Build index and generate views** (0.944) - src/cli/build.rs

### Most Diverse

- **context-map** (241) - README.md
- **Core data types for the indexer** (116) - src/types.rs
- **context-map: Deterministic knowledge base indexer for AI agents** (58) - src/main.rs
- **Configuration types and defaults** (49) - src/config/mod.rs
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** (25) - src/analyze/mod.rs
- **Text extraction from various file types** (23) - src/extract/mod.rs
- **Unicode-aware word tokenization** (21) - src/analyze/tokenize.rs
- **Global aggregation: term index, folder signatures** (17) - src/aggregate/mod.rs
- **File fingerprinting for change detection** (17) - src/scan/fingerprint.rs
- **Markdown view rendering** (14) - src/render/mod.rs

## Top Concepts

- command
- cmap
- content
- extraction
- cache
- extract
- features
- management
- text
- generate
- compute
- generation
- fingerprints
- keyword
- tree
- folder
- sitter
- terms.md
- root_atlas.md
- computation
- yake
- markdown
- fingerprint
- cli
- directory
- all
- term
- cached
- build
- scan

## Navigation

- Each folder has an `INDEX.md` with detailed file listings
- See `TERMS.md` for concept-to-file mappings
- File paths are relative to the knowledge base root
