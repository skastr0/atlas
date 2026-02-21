# Term Index

_Mapping of key terms and phrases to files. Use this to find files about specific topics._

## command

_Found in 6 files_

- **CLI command implementations** — `src/cli/mod.rs`
- **`cmap clean` command - Remove cached data** — `src/cli/clean.rs`
- **`cmap doctor` command - Report issues** — `src/cli/doctor.rs`
- **`cmap init` command - Initialize .cmap directory** — `src/cli/init.rs`
- **`cmap scan` command - Scan files and update fingerprints** — `src/cli/scan.rs`
- **`cmap build` command - Build index and generate views** — `src/cli/build.rs`

## cmap

_Found in 6 files_

- **`cmap init` command - Initialize .cmap directory** — `src/cli/init.rs`
- **`cmap clean` command - Remove cached data** — `src/cli/clean.rs`
- **`cmap doctor` command - Report issues** — `src/cli/doctor.rs`
- **`cmap scan` command - Scan files and update fingerprints** — `src/cli/scan.rs`
- **`cmap build` command - Build index and generate views** — `src/cli/build.rs`
- **context-map** — `README.md`

## content

_Found in 9 files_

- **ROOT_ATLAS.md generation** — `src/render/atlas.rs`
- **TERMS.md generation** — `src/render/term_index.rs`
- **Markdown text extraction** — `src/extract/markdown.rs`
- **TypeScript and TSX extraction using tree-sitter** — `src/extract/typescript.rs`
- **Plain text extraction** — `src/extract/plaintext.rs`
- **Text extraction from various file types** — `src/extract/mod.rs`
- **Rust source extraction using tree-sitter** — `src/extract/rust.rs`
- **Per-folder INDEX.md generation** — `src/render/folder_index.rs`
- **File fingerprinting for change detection** — `src/scan/fingerprint.rs`

## extraction

_Found in 11 files_

- **YAKE keyword extraction** — `src/analyze/yake.rs`
- **Markdown text extraction** — `src/extract/markdown.rs`
- **TypeScript and TSX extraction using tree-sitter** — `src/extract/typescript.rs`
- **Plain text extraction** — `src/extract/plaintext.rs`
- **Link and citation extraction** — `src/analyze/links.rs`
- **Rust source extraction using tree-sitter** — `src/extract/rust.rs`
- **RAKE (Rapid Automatic Keyword Extraction)** — `src/analyze/rake.rs`
- **N-gram extraction (bigrams, trigrams)** — `src/analyze/ngrams.rs`
- **PDF text extraction via pdftotext** — `src/extract/pdf.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`

## cache

_Found in 4 files_

- **Fingerprint cache management** — `src/cache/fingerprints.rs`
- **Cache management for fingerprints and features** — `src/cache/mod.rs`
- **Feature cache management** — `src/cache/features.rs`
- **context-map: Deterministic knowledge base indexer for AI agents** — `src/main.rs`

## extract

_Found in 8 files_

- **Markdown text extraction** — `src/extract/markdown.rs`
- **N-gram extraction (bigrams, trigrams)** — `src/analyze/ngrams.rs`
- **TypeScript and TSX extraction using tree-sitter** — `src/extract/typescript.rs`
- **Plain text extraction** — `src/extract/plaintext.rs`
- **Link and citation extraction** — `src/analyze/links.rs`
- **Rust source extraction using tree-sitter** — `src/extract/rust.rs`
- **PDF text extraction via pdftotext** — `src/extract/pdf.rs`
- **Text extraction from various file types** — `src/extract/mod.rs`

## features

_Found in 4 files_

- **Cache management for fingerprints and features** — `src/cache/mod.rs`
- **Per-file feature computation** — `src/analyze/features.rs`
- **Feature cache management** — `src/cache/features.rs`
- **Global term index and document frequency** — `src/aggregate/term_index.rs`

## management

_Found in 3 files_

- **Fingerprint cache management** — `src/cache/fingerprints.rs`
- **Cache management for fingerprints and features** — `src/cache/mod.rs`
- **Feature cache management** — `src/cache/features.rs`

## text

_Found in 6 files_

- **Plain text extraction** — `src/extract/plaintext.rs`
- **Markdown text extraction** — `src/extract/markdown.rs`
- **PDF text extraction via pdftotext** — `src/extract/pdf.rs`
- **Text extraction from various file types** — `src/extract/mod.rs`
- **Link and citation extraction** — `src/analyze/links.rs`
- **Unicode-aware word tokenization** — `src/analyze/tokenize.rs`

## generate

_Found in 4 files_

- **ROOT_ATLAS.md generation** — `src/render/atlas.rs`
- **TERMS.md generation** — `src/render/term_index.rs`
- **Per-folder INDEX.md generation** — `src/render/folder_index.rs`
- **`cmap build` command - Build index and generate views** — `src/cli/build.rs`

## compute

_Found in 4 files_

- **Per-file feature computation** — `src/analyze/features.rs`
- **TF-IDF computation** — `src/analyze/tfidf.rs`
- **Folder signature computation** — `src/aggregate/folder_sig.rs`
- **File fingerprinting for change detection** — `src/scan/fingerprint.rs`

## generation

_Found in 5 files_

- **ROOT_ATLAS.md generation** — `src/render/atlas.rs`
- **TERMS.md generation** — `src/render/term_index.rs`
- **CONNECTIONS.md and graph generation** — `src/render/graph.rs`
- **Per-folder INDEX.md generation** — `src/render/folder_index.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`

## fingerprints

_Found in 5 files_

- **Cache management for fingerprints and features** — `src/cache/mod.rs`
- **`cmap scan` command - Scan files and update fingerprints** — `src/cli/scan.rs`
- **File fingerprinting for change detection** — `src/scan/fingerprint.rs`
- **File scanning and fingerprinting** — `src/scan/mod.rs`
- **Core data types for the indexer** — `src/types.rs`

## keyword

_Found in 2 files_

- **YAKE keyword extraction** — `src/analyze/yake.rs`
- **RAKE (Rapid Automatic Keyword Extraction)** — `src/analyze/rake.rs`

## tree

_Found in 4 files_

- **use crate::types::FileType;** — `src/extract/treesitter.rs`
- **TypeScript and TSX extraction using tree-sitter** — `src/extract/typescript.rs`
- **Rust source extraction using tree-sitter** — `src/extract/rust.rs`
- **Directory traversal with ignore patterns** — `src/scan/walker.rs`

## folder

_Found in 7 files_

- **Per-folder INDEX.md generation** — `src/render/folder_index.rs`
- **Folder signature computation** — `src/aggregate/folder_sig.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`
- **Markdown view rendering** — `src/render/mod.rs`
- **Configuration types and defaults** — `src/config/mod.rs`
- **Core data types for the indexer** — `src/types.rs`
- **context-map** — `README.md`

## sitter

_Found in 3 files_

- **use crate::types::FileType;** — `src/extract/treesitter.rs`
- **TypeScript and TSX extraction using tree-sitter** — `src/extract/typescript.rs`
- **Rust source extraction using tree-sitter** — `src/extract/rust.rs`

## terms.md

_Found in 2 files_

- **TERMS.md generation** — `src/render/term_index.rs`
- **Markdown view rendering** — `src/render/mod.rs`

## root_atlas.md

_Found in 3 files_

- **ROOT_ATLAS.md generation** — `src/render/atlas.rs`
- **Markdown view rendering** — `src/render/mod.rs`
- **context-map** — `README.md`

## computation

_Found in 5 files_

- **Per-file feature computation** — `src/analyze/features.rs`
- **Folder signature computation** — `src/aggregate/folder_sig.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`
- **TF-IDF computation** — `src/analyze/tfidf.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`

## yake

_Found in 2 files_

- **YAKE keyword extraction** — `src/analyze/yake.rs`
- **Core data types for the indexer** — `src/types.rs`

## markdown

_Found in 4 files_

- **Markdown text extraction** — `src/extract/markdown.rs`
- **Markdown view rendering** — `src/render/mod.rs`
- **Text extraction from various file types** — `src/extract/mod.rs`
- **context-map** — `README.md`

## fingerprint

_Found in 2 files_

- **Fingerprint cache management** — `src/cache/fingerprints.rs`
- **File fingerprinting for change detection** — `src/scan/fingerprint.rs`

## cli

_Found in 1 files_

- **CLI command implementations** — `src/cli/mod.rs`

## directory

_Found in 4 files_

- **`cmap init` command - Initialize .cmap directory** — `src/cli/init.rs`
- **Directory traversal with ignore patterns** — `src/scan/walker.rs`
- **File scanning and fingerprinting** — `src/scan/mod.rs`
- **context-map: Deterministic knowledge base indexer for AI agents** — `src/main.rs`

## all

_Found in 5 files_

- **Folder signature computation** — `src/aggregate/folder_sig.rs`
- **Link and citation extraction** — `src/analyze/links.rs`
- **Per-folder INDEX.md generation** — `src/render/folder_index.rs`
- **Directory traversal with ignore patterns** — `src/scan/walker.rs`
- **Feature cache management** — `src/cache/features.rs`

## term

_Found in 6 files_

- **Global term index and document frequency** — `src/aggregate/term_index.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`
- **TF-IDF computation** — `src/analyze/tfidf.rs`
- **Markdown view rendering** — `src/render/mod.rs`
- **Configuration types and defaults** — `src/config/mod.rs`
- **Core data types for the indexer** — `src/types.rs`

## cached

_Found in 4 files_

- **`cmap clean` command - Remove cached data** — `src/cli/clean.rs`
- **Feature cache management** — `src/cache/features.rs`
- **File scanning and fingerprinting** — `src/scan/mod.rs`
- **File fingerprinting for change detection** — `src/scan/fingerprint.rs`

## build

_Found in 3 files_

- **`cmap build` command - Build index and generate views** — `src/cli/build.rs`
- **Global term index and document frequency** — `src/aggregate/term_index.rs`
- **context-map** — `README.md`

## scan

_Found in 2 files_

- **`cmap scan` command - Scan files and update fingerprints** — `src/cli/scan.rs`
- **context-map** — `README.md`

## idf

_Found in 5 files_

- **TF-IDF computation** — `src/analyze/tfidf.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`
- **Global term index and document frequency** — `src/aggregate/term_index.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`
- **Core data types for the indexer** — `src/types.rs`

## per

_Found in 4 files_

- **Per-file feature computation** — `src/analyze/features.rs`
- **Per-folder INDEX.md generation** — `src/render/folder_index.rs`
- **Markdown view rendering** — `src/render/mod.rs`
- **Configuration types and defaults** — `src/config/mod.rs`

## plain

_Found in 2 files_

- **Plain text extraction** — `src/extract/plaintext.rs`
- **Text extraction from various file types** — `src/extract/mod.rs`

## pdftotext

_Found in 2 files_

- **PDF text extraction via pdftotext** — `src/extract/pdf.rs`
- **Text extraction from various file types** — `src/extract/mod.rs`

## source

_Found in 2 files_

- **Rust source extraction using tree-sitter** — `src/extract/rust.rs`
- **use crate::types::FileType;** — `src/extract/treesitter.rs`

## index

_Found in 3 files_

- **Global term index and document frequency** — `src/aggregate/term_index.rs`
- **`cmap build` command - Build index and generate views** — `src/cli/build.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`

## global

_Found in 4 files_

- **Global term index and document frequency** — `src/aggregate/term_index.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`
- **Core data types for the indexer** — `src/types.rs`

## feature

_Found in 2 files_

- **Per-file feature computation** — `src/analyze/features.rs`
- **Feature cache management** — `src/cache/features.rs`

## link

_Found in 4 files_

- **Link and citation extraction** — `src/analyze/links.rs`
- **CONNECTIONS.md and graph generation** — `src/render/graph.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`
- **Core data types for the indexer** — `src/types.rs`

## remove

_Found in 3 files_

- **`cmap clean` command - Remove cached data** — `src/cli/clean.rs`
- **Feature cache management** — `src/cache/features.rs`
- **context-map: Deterministic knowledge base indexer for AI agents** — `src/main.rs`

## document

_Found in 5 files_

- **TF-IDF computation** — `src/analyze/tfidf.rs`
- **Global term index and document frequency** — `src/aggregate/term_index.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`
- **Configuration types and defaults** — `src/config/mod.rs`
- **Core data types for the indexer** — `src/types.rs`

## index.md

_Found in 3 files_

- **Per-folder INDEX.md generation** — `src/render/folder_index.rs`
- **Markdown view rendering** — `src/render/mod.rs`
- **context-map** — `README.md`

## update

_Found in 3 files_

- **`cmap scan` command - Scan files and update fingerprints** — `src/cli/scan.rs`
- **Global term index and document frequency** — `src/aggregate/term_index.rs`
- **context-map: Deterministic knowledge base indexer for AI agents** — `src/main.rs`

## links

_Found in 2 files_

- **Link and citation extraction** — `src/analyze/links.rs`
- **Text extraction from various file types** — `src/extract/mod.rs`

## patterns

_Found in 2 files_

- **Directory traversal with ignore patterns** — `src/scan/walker.rs`
- **File scanning and fingerprinting** — `src/scan/mod.rs`

## clean

_Found in 1 files_

- **`cmap clean` command - Remove cached data** — `src/cli/clean.rs`

## doctor

_Found in 1 files_

- **`cmap doctor` command - Report issues** — `src/cli/doctor.rs`

## top

_Found in 6 files_

- **N-gram extraction (bigrams, trigrams)** — `src/analyze/ngrams.rs`
- **Markdown view rendering** — `src/render/mod.rs`
- **TF-IDF computation** — `src/analyze/tfidf.rs`
- **Core data types for the indexer** — `src/types.rs`
- **Configuration types and defaults** — `src/config/mod.rs`
- **context-map** — `README.md`

## list

_Found in 2 files_

- **CONNECTIONS.md and graph generation** — `src/render/graph.rs`
- **Configuration types and defaults** — `src/config/mod.rs`

## fingerprinting

_Found in 2 files_

- **File scanning and fingerprinting** — `src/scan/mod.rs`
- **File fingerprinting for change detection** — `src/scan/fingerprint.rs`

## tokens

_Found in 2 files_

- **N-gram extraction (bigrams, trigrams)** — `src/analyze/ngrams.rs`
- **Unicode-aware word tokenization** — `src/analyze/tokenize.rs`

## ignore

_Found in 2 files_

- **Directory traversal with ignore patterns** — `src/scan/walker.rs`
- **File scanning and fingerprinting** — `src/scan/mod.rs`

## issues

_Found in 1 files_

- **`cmap doctor` command - Report issues** — `src/cli/doctor.rs`

## report

_Found in 1 files_

- **`cmap doctor` command - Report issues** — `src/cli/doctor.rs`

## signatures

_Found in 2 files_

- **Folder signature computation** — `src/aggregate/folder_sig.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`

## signature

_Found in 2 files_

- **Folder signature computation** — `src/aggregate/folder_sig.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`

## init

_Found in 2 files_

- **`cmap init` command - Initialize .cmap directory** — `src/cli/init.rs`
- **context-map** — `README.md`

## path

_Found in 2 files_

- **CONNECTIONS.md and graph generation** — `src/render/graph.rs`
- **Core data types for the indexer** — `src/types.rs`

## count

_Found in 3 files_

- **Link and citation extraction** — `src/analyze/links.rs`
- **N-gram extraction (bigrams, trigrams)** — `src/analyze/ngrams.rs`
- **Core data types for the indexer** — `src/types.rs`

## views

_Found in 3 files_

- **`cmap build` command - Build index and generate views** — `src/cli/build.rs`
- **context-map: Deterministic knowledge base indexer for AI agents** — `src/main.rs`
- **context-map** — `README.md`

## configuration

_Found in 3 files_

- **Configuration types and defaults** — `src/config/mod.rs`
- **Directory traversal with ignore patterns** — `src/scan/walker.rs`
- **Unicode-aware word tokenization** — `src/analyze/tokenize.rs`

## map

_Found in 3 files_

- **context-map library** — `src/lib.rs`
- **Markdown view rendering** — `src/render/mod.rs`
- **context-map** — `README.md`

## knowledge

_Found in 4 files_

- **context-map library** — `src/lib.rs`
- **context-map: Deterministic knowledge base indexer for AI agents** — `src/main.rs`
- **context-map** — `README.md`
- **Core data types for the indexer** — `src/types.rs`

## initialize

_Found in 1 files_

- **`cmap init` command - Initialize .cmap directory** — `src/cli/init.rs`

## frequency

_Found in 5 files_

- **Global term index and document frequency** — `src/aggregate/term_index.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`
- **Configuration types and defaults** — `src/config/mod.rs`
- **Core data types for the indexer** — `src/types.rs`

## load

_Found in 2 files_

- **Feature cache management** — `src/cache/features.rs`
- **File fingerprinting for change detection** — `src/scan/fingerprint.rs`

## traversal

_Found in 2 files_

- **Directory traversal with ignore patterns** — `src/scan/walker.rs`
- **File scanning and fingerprinting** — `src/scan/mod.rs`

## folders

_Found in 1 files_

- **Folder signature computation** — `src/aggregate/folder_sig.rs`

## module

_Found in 3 files_

- **File scanning and fingerprinting** — `src/scan/mod.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`

## handles

_Found in 3 files_

- **File scanning and fingerprinting** — `src/scan/mod.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`

## trigrams

_Found in 1 files_

- **N-gram extraction (bigrams, trigrams)** — `src/analyze/ngrams.rs`

## bigrams

_Found in 1 files_

- **N-gram extraction (bigrams, trigrams)** — `src/analyze/ngrams.rs`

## base

_Found in 4 files_

- **context-map library** — `src/lib.rs`
- **context-map: Deterministic knowledge base indexer for AI agents** — `src/main.rs`
- **Core data types for the indexer** — `src/types.rs`
- **context-map** — `README.md`

## citation

_Found in 2 files_

- **Link and citation extraction** — `src/analyze/links.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`

## stopword

_Found in 2 files_

- **RAKE (Rapid Automatic Keyword Extraction)** — `src/analyze/rake.rs`
- **Unicode-aware word tokenization** — `src/analyze/tokenize.rs`

## stopwords

_Found in 2 files_

- **Unicode-aware word tokenization** — `src/analyze/tokenize.rs`
- **Configuration types and defaults** — `src/config/mod.rs`

## scores

_Found in 2 files_

- **Global term index and document frequency** — `src/aggregate/term_index.rs`
- **TF-IDF computation** — `src/analyze/tfidf.rs`

## grams

_Found in 2 files_

- **N-gram extraction (bigrams, trigrams)** — `src/analyze/ngrams.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`

## gram

_Found in 2 files_

- **N-gram extraction (bigrams, trigrams)** — `src/analyze/ngrams.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`

## matching

_Found in 2 files_

- **Directory traversal with ignore patterns** — `src/scan/walker.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`

## supported

_Found in 1 files_

- **use crate::types::FileType;** — `src/extract/treesitter.rs`

## core

_Found in 1 files_

- **context-map library** — `src/lib.rs`

## types

_Found in 2 files_

- **context-map library** — `src/lib.rs`
- **Text extraction from various file types** — `src/extract/mod.rs`

## context

_Found in 2 files_

- **context-map library** — `src/lib.rs`
- **context-map** — `README.md`

## change

_Found in 2 files_

- **File scanning and fingerprinting** — `src/scan/mod.rs`
- **File fingerprinting for change detection** — `src/scan/fingerprint.rs`

## detection

_Found in 2 files_

- **File scanning and fingerprinting** — `src/scan/mod.rs`
- **File fingerprinting for change detection** — `src/scan/fingerprint.rs`

## save

_Found in 2 files_

- **Feature cache management** — `src/cache/features.rs`
- **File fingerprinting for change detection** — `src/scan/fingerprint.rs`

## tokenization

_Found in 2 files_

- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`
- **Unicode-aware word tokenization** — `src/analyze/tokenize.rs`

## via

_Found in 2 files_

- **PDF text extraction via pdftotext** — `src/extract/pdf.rs`
- **Text extraction from various file types** — `src/extract/mod.rs`

## check

_Found in 2 files_

- **PDF text extraction via pdftotext** — `src/extract/pdf.rs`
- **Unicode-aware word tokenization** — `src/analyze/tokenize.rs`

## extracts

_Found in 1 files_

- **RAKE (Rapid Automatic Keyword Extraction)** — `src/analyze/rake.rs`

## rake

_Found in 1 files_

- **RAKE (Rapid Automatic Keyword Extraction)** — `src/analyze/rake.rs`

## scoring

_Found in 2 files_

- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`

## word

_Found in 2 files_

- **Unicode-aware word tokenization** — `src/analyze/tokenize.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`

## score

_Found in 2 files_

- **TF-IDF computation** — `src/analyze/tfidf.rs`
- **Core data types for the indexer** — `src/types.rs`

## available

_Found in 1 files_

- **PDF text extraction via pdftotext** — `src/extract/pdf.rs`

## rendering

_Found in 2 files_

- **Markdown view rendering** — `src/render/mod.rs`
- **Configuration types and defaults** — `src/config/mod.rs`

## length

_Found in 2 files_

- **Configuration types and defaults** — `src/config/mod.rs`
- **Unicode-aware word tokenization** — `src/analyze/tokenize.rs`

## phrases

_Found in 1 files_

- **RAKE (Rapid Automatic Keyword Extraction)** — `src/analyze/rake.rs`

## indexes

_Found in 1 files_

- **Per-folder INDEX.md generation** — `src/render/folder_index.rs`

---

## Key Phrases

### knowledge base

_Found in 4 files_

- **context-map** — `README.md`
- **Core data types for the indexer** — `src/types.rs`
- **context-map: Deterministic knowledge base indexer for AI agents** — `src/main.rs`
- **context-map library** — `src/lib.rs`

### folder index.md

_Found in 3 files_

- **context-map** — `README.md`
- **Per-folder INDEX.md generation** — `src/render/folder_index.rs`
- **Markdown view rendering** — `src/render/mod.rs`

### text extraction

_Found in 4 files_

- **PDF text extraction via pdftotext** — `src/extract/pdf.rs`
- **Plain text extraction** — `src/extract/plaintext.rs`
- **Text extraction from various file types** — `src/extract/mod.rs`
- **Markdown text extraction** — `src/extract/markdown.rs`

### relative path

_Found in 1 files_

- **Core data types for the indexer** — `src/types.rs`

### knowledge base root

_Found in 1 files_

- **Core data types for the indexer** — `src/types.rs`

### tree sitter

_Found in 2 files_

- **use crate::types::FileType;** — `src/extract/treesitter.rs`
- **TypeScript and TSX extraction using tree-sitter** — `src/extract/typescript.rs`

### path knowledge base

_Found in 1 files_

- **Core data types for the indexer** — `src/types.rs`

### extract content

_Found in 3 files_

- **TypeScript and TSX extraction using tree-sitter** — `src/extract/typescript.rs`
- **Plain text extraction** — `src/extract/plaintext.rs`
- **Markdown text extraction** — `src/extract/markdown.rs`

### global term

_Found in 2 files_

- **Global term index and document frequency** — `src/aggregate/term_index.rs`
- **Global aggregation: term index, folder signatures** — `src/aggregate/mod.rs`

### computation compute

_Found in 3 files_

- **Folder signature computation** — `src/aggregate/folder_sig.rs`
- **Per-file feature computation** — `src/analyze/features.rs`
- **TF-IDF computation** — `src/analyze/tfidf.rs`

### build release

_Found in 1 files_

- **context-map** — `README.md`

### cache management

_Found in 3 files_

- **Feature cache management** — `src/cache/features.rs`
- **Cache management for fingerprints and features** — `src/cache/mod.rs`
- **Fingerprint cache management** — `src/cache/fingerprints.rs`

### relative path knowledge

_Found in 1 files_

- **Core data types for the indexer** — `src/types.rs`

### top terms

_Found in 1 files_

- **context-map** — `README.md`

### base root

_Found in 1 files_

- **Core data types for the indexer** — `src/types.rs`

### path knowledge

_Found in 1 files_

- **Core data types for the indexer** — `src/types.rs`

### extraction extract

_Found in 3 files_

- **Link and citation extraction** — `src/analyze/links.rs`
- **Plain text extraction** — `src/extract/plaintext.rs`
- **Markdown text extraction** — `src/extract/markdown.rs`

### cargo build release

_Found in 1 files_

- **context-map** — `README.md`

### cargo build

_Found in 1 files_

- **context-map** — `README.md`

### install poppler

_Found in 1 files_

- **context-map** — `README.md`

### length maximum

_Found in 1 files_

- **Configuration types and defaults** — `src/config/mod.rs`

### global term index

_Found in 1 files_

- **Global term index and document frequency** — `src/aggregate/term_index.rs`

### document frequency

_Found in 1 files_

- **Configuration types and defaults** — `src/config/mod.rs`

### tree sitter extract

_Found in 2 files_

- **TypeScript and TSX extraction using tree-sitter** — `src/extract/typescript.rs`
- **Rust source extraction using tree-sitter** — `src/extract/rust.rs`

### fingerprints jsonl

_Found in 1 files_

- **File fingerprinting for change detection** — `src/scan/fingerprint.rs`

### path list

_Found in 1 files_

- **CONNECTIONS.md and graph generation** — `src/render/graph.rs`

### generation generate

_Found in 2 files_

- **ROOT_ATLAS.md generation** — `src/render/atlas.rs`
- **TERMS.md generation** — `src/render/term_index.rs`

### term length

_Found in 1 files_

- **Configuration types and defaults** — `src/config/mod.rs`

### install poppler utils

_Found in 1 files_

- **context-map** — `README.md`

### folder index.md detailed

_Found in 1 files_

- **context-map** — `README.md`

### cached features

_Found in 1 files_

- **Feature cache management** — `src/cache/features.rs`

### link citation

_Found in 2 files_

- **Link and citation extraction** — `src/analyze/links.rs`
- **Text analysis: tokenization, n-grams, TF-IDF, link extraction** — `src/analyze/mod.rs`

### change detection

_Found in 2 files_

- **File fingerprinting for change detection** — `src/scan/fingerprint.rs`
- **File scanning and fingerprinting** — `src/scan/mod.rs`

### rust source

_Found in 1 files_

- **Rust source extraction using tree-sitter** — `src/extract/rust.rs`

### term length maximum

_Found in 1 files_

- **Configuration types and defaults** — `src/config/mod.rs`

### extraction extract content

_Found in 2 files_

- **Plain text extraction** — `src/extract/plaintext.rs`
- **Markdown text extraction** — `src/extract/markdown.rs`

### basic english

_Found in 1 files_

- **Unicode-aware word tokenization** — `src/analyze/tokenize.rs`

### extraction tree sitter

_Found in 2 files_

- **TypeScript and TSX extraction using tree-sitter** — `src/extract/typescript.rs`
- **Rust source extraction using tree-sitter** — `src/extract/rust.rs`

### term index

_Found in 1 files_

- **Global term index and document frequency** — `src/aggregate/term_index.rs`

### compute idf

_Found in 1 files_

- **TF-IDF computation** — `src/analyze/tfidf.rs`

### extraction tree

_Found in 2 files_

- **TypeScript and TSX extraction using tree-sitter** — `src/extract/typescript.rs`
- **Rust source extraction using tree-sitter** — `src/extract/rust.rs`

### detailed listings

_Found in 1 files_

- **context-map** — `README.md`

### idf score

_Found in 1 files_

- **Core data types for the indexer** — `src/types.rs`

### index generate

_Found in 2 files_

- **`cmap build` command - Build index and generate views** — `src/cli/build.rs`
- **context-map: Deterministic knowledge base indexer for AI agents** — `src/main.rs`

### unix timestamp

_Found in 1 files_

- **Core data types for the indexer** — `src/types.rs`

### text extraction extract

_Found in 2 files_

- **Plain text extraction** — `src/extract/plaintext.rs`
- **Markdown text extraction** — `src/extract/markdown.rs`

### tokenize filter

_Found in 1 files_

- **Unicode-aware word tokenization** — `src/analyze/tokenize.rs`

### keyword extraction

_Found in 2 files_

- **YAKE keyword extraction** — `src/analyze/yake.rs`
- **RAKE (Rapid Automatic Keyword Extraction)** — `src/analyze/rake.rs`

### sitter extract

_Found in 2 files_

- **TypeScript and TSX extraction using tree-sitter** — `src/extract/typescript.rs`
- **Rust source extraction using tree-sitter** — `src/extract/rust.rs`

### initialize cmap

_Found in 2 files_

- **`cmap init` command - Initialize .cmap directory** — `src/cli/init.rs`
- **context-map: Deterministic knowledge base indexer for AI agents** — `src/main.rs`

