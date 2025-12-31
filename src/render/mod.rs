//! Markdown view rendering
//!
//! Generates:
//! - ROOT_ATLAS.md - Top-level map
//! - Per-folder INDEX.md files
//! - TERMS.md - Term to files mapping

mod atlas;
mod folder_index;
mod graph;
mod term_index;

pub use atlas::*;
pub use folder_index::*;
pub use graph::*;
pub use term_index::*;
