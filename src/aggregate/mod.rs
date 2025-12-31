//! Global aggregation: term index, folder signatures
//!
//! This module handles:
//! - Global term document frequency computation
//! - TF-IDF scoring across corpus
//! - Folder signature generation

mod folder_sig;
mod term_index;

pub use folder_sig::*;
pub use term_index::*;
