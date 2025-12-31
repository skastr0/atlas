//! Plain text extraction

use super::ExtractedContent;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Extract content from plain text file
pub fn extract_plaintext(path: &Path) -> Result<ExtractedContent> {
    let text = fs::read_to_string(path)?;

    // Try to extract title from first line if it looks like a title
    let title = text
        .lines()
        .next()
        .filter(|line| !line.is_empty() && line.len() < 200)
        .map(|s| s.to_string());

    Ok(ExtractedContent {
        text,
        title,
        headings: Vec::new(),
        links: Vec::new(),
        success: true,
    })
}
