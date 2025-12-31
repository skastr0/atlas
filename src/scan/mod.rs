//! File scanning and fingerprinting
//!
//! This module handles:
//! - Directory traversal with ignore patterns
//! - File fingerprinting for change detection
//! - Comparison with cached fingerprints

mod fingerprint;
mod walker;

pub use fingerprint::*;
pub use walker::*;
