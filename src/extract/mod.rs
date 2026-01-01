//! Text extraction from various file types
//!
//! Supports:
//! - Markdown (.md)
//! - Plain text (.txt)
//! - PDF (.pdf) via pdftotext
//! - reStructuredText (.rst)
//! - Org mode (.org)

mod markdown;
mod pdf;
mod plaintext;
mod rust;
mod treesitter;
mod typescript;

pub use markdown::*;
pub use pdf::*;
pub use plaintext::*;
pub use rust::*;
pub use treesitter::*;
pub use typescript::*;

use crate::config::ExtractConfig;
use crate::types::FileType;
use anyhow::Result;
use std::path::Path;

/// Extracted content from a file
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    /// Raw text content
    pub text: String,
    /// Title (if extractable)
    pub title: Option<String>,
    /// Headings (if extractable)
    pub headings: Vec<String>,
    /// Links found in content
    pub links: Vec<String>,
    /// Whether extraction succeeded fully
    pub success: bool,
}

/// Extract text content from a file
pub fn extract(path: &Path, file_type: FileType, config: &ExtractConfig) -> Result<ExtractedContent> {
    match file_type {
        FileType::Markdown => extract_markdown(path),
        FileType::PlainText | FileType::Rst | FileType::Org => extract_plaintext(path),
        FileType::Pdf => extract_pdf(path, config),
        // Code files: use tree-sitter where available
        FileType::Rust => extract_rust(path),
        FileType::TypeScript => extract_typescript(path, false),
        FileType::Tsx => extract_typescript(path, true),
        FileType::Unknown => extract_plaintext(path),
    }
}
