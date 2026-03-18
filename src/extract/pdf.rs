//! PDF text extraction via pdftotext

use super::ExtractedContent;
use crate::config::ExtractConfig;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Extract text from PDF using pdftotext
pub fn extract_pdf(path: &Path, config: &ExtractConfig) -> Result<ExtractedContent> {
    // Try to find pdftotext
    let pdftotext = resolve_pdftotext(config.pdftotext_path.as_deref())?;

    // Run pdftotext with output to stdout
    let output = Command::new(&pdftotext)
        .arg("-layout") // Preserve layout for better readability
        .arg(path)
        .arg("-") // Output to stdout
        .output()
        .context("Failed to run pdftotext")?;

    if !output.status.success() {
        let _stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(ExtractedContent {
            text: String::new(),
            title: None,
            headings: Vec::new(),
            links: Vec::new(),
            success: false,
        });
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();

    // Try to extract title from first non-empty lines
    let title = text
        .lines()
        .find(|line| !line.trim().is_empty())
        .filter(|line| line.len() < 200)
        .map(|s| s.trim().to_string());

    Ok(ExtractedContent {
        text,
        title,
        headings: Vec::new(), // Hard to extract reliably from PDFs
        links: Vec::new(),    // Would need more sophisticated parsing
        success: true,
    })
}

/// Find pdftotext binary
pub fn resolve_pdftotext(custom_path: Option<&str>) -> Result<String> {
    if let Some(custom_path) = custom_path {
        if Path::new(custom_path).exists() {
            return Ok(custom_path.to_string());
        }

        if Command::new("which")
            .arg(custom_path)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Ok(custom_path.to_string());
        }

        anyhow::bail!("pdftotext_path not found: {}", custom_path)
    }

    // Common locations
    let candidates = [
        "pdftotext",
        "/usr/bin/pdftotext",
        "/usr/local/bin/pdftotext",
        "/opt/homebrew/bin/pdftotext",
    ];

    for candidate in candidates {
        if Command::new("which")
            .arg(candidate)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Ok(candidate.to_string());
        }

        // Direct check for absolute paths
        if candidate.starts_with('/') && std::path::Path::new(candidate).exists() {
            return Ok(candidate.to_string());
        }
    }

    anyhow::bail!(
        "pdftotext not found. Install poppler-utils:\n  \
        macOS: brew install poppler\n  \
        Ubuntu/Debian: apt install poppler-utils\n  \
        Fedora: dnf install poppler-utils"
    )
}

/// Check if pdftotext is available
pub fn is_pdftotext_available() -> bool {
    resolve_pdftotext(None).is_ok()
}
