# Knowledge Base Connections

_Auto-generated map of internal file connections. Use this to find hubs and orphans._

## Hub Files (Most Referenced)

- **src/types.rs** (15 inbound links)
- **src/config/mod.rs** (11 inbound links)
- **src/analyze/tokenize.rs** (2 inbound links)
- **src/extract/mod.rs** (2 inbound links)
- **src/extract/treesitter.rs** (2 inbound links)
- **src/scan/mod.rs** (2 inbound links)
- **src/aggregate/mod.rs** (1 inbound links)
- **src/analyze/mod.rs** (1 inbound links)
- **src/cache/mod.rs** (1 inbound links)
- **src/render/mod.rs** (1 inbound links)

## Orphan Files (No Connections)

- README.md
- src/analyze/tfidf.rs
- src/cli/clean.rs
- src/cli/doctor.rs
- src/cli/mod.rs
- src/cli/scan.rs
- src/extract/markdown.rs
- src/extract/plaintext.rs
- src/lib.rs
- src/main.rs

## Connection Graph

```mermaid
graph LR
    n1["src/aggregate/folder_sig.rs"] --> n22["src/types.rs"]
    n3["src/aggregate/term_index.rs"] --> n22["src/types.rs"]
    n4["src/analyze/features.rs"] --> n22["src/types.rs"]
    n5["src/analyze/links.rs"] --> n22["src/types.rs"]
    n8["src/analyze/rake.rs"] --> n22["src/types.rs"]
    n10["src/analyze/yake.rs"] --> n22["src/types.rs"]
    n4["src/analyze/features.rs"] --> n15["src/config/mod.rs"]
    n8["src/analyze/rake.rs"] --> n15["src/config/mod.rs"]
    n9["src/analyze/tokenize.rs"] --> n15["src/config/mod.rs"]
    n10["src/analyze/yake.rs"] --> n15["src/config/mod.rs"]
    n13["src/cli/build.rs"] --> n15["src/config/mod.rs"]
    n14["src/cli/init.rs"] --> n15["src/config/mod.rs"]
    n7["src/analyze/ngrams.rs"] --> n9["src/analyze/tokenize.rs"]
    n10["src/analyze/yake.rs"] --> n9["src/analyze/tokenize.rs"]
    n4["src/analyze/features.rs"] --> n16["src/extract/mod.rs"]
    n13["src/cli/build.rs"] --> n16["src/extract/mod.rs"]
    n17["src/extract/rust.rs"] --> n18["src/extract/treesitter.rs"]
    n19["src/extract/typescript.rs"] --> n18["src/extract/treesitter.rs"]
    n11["src/cache/fingerprints.rs"] --> n21["src/scan/mod.rs"]
    n13["src/cli/build.rs"] --> n21["src/scan/mod.rs"]
    n13["src/cli/build.rs"] --> n2["src/aggregate/mod.rs"]
    n13["src/cli/build.rs"] --> n6["src/analyze/mod.rs"]
    n13["src/cli/build.rs"] --> n12["src/cache/mod.rs"]
    n13["src/cli/build.rs"] --> n20["src/render/mod.rs"]
```
