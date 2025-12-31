//! Text analysis: tokenization, n-grams, TF-IDF, link extraction
//!
//! This module handles:
//! - Unicode-aware word tokenization
//! - N-gram (bigram, trigram) extraction
//! - Term frequency computation
//! - TF-IDF scoring (requires global pass)
//! - Link/citation pattern matching

mod features;
mod links;
mod ngrams;
mod rake;
mod tfidf;
mod tokenize;
mod yake;

pub use features::*;
pub use links::*;
pub use ngrams::*;
pub use rake::*;
pub use tfidf::*;
pub use tokenize::*;
pub use yake::*;
