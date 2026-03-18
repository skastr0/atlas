//! Text extraction from various file types
//!
//! Supports:
//! - Markdown (.md)
//! - Plain text and common config/text files (.txt, .json, .yml, .yaml, .toml, .sh, .sql)
//! - PDF (.pdf) via pdftotext
//! - reStructuredText (.rst)
//! - Org mode (.org)
//! - JavaScript/TypeScript (.js, .mjs, .cjs, .ts)
//! - JSX/TSX (.jsx, .tsx)

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
pub fn extract(
    path: &Path,
    file_type: FileType,
    config: &ExtractConfig,
) -> Result<ExtractedContent> {
    match file_type {
        FileType::Markdown => extract_markdown(path),
        FileType::PlainText | FileType::Rst | FileType::Org => extract_plaintext(path),
        FileType::Pdf => extract_pdf(path, config),
        // Code files: use tree-sitter where available
        FileType::Rust => extract_rust(path),
        FileType::JavaScript | FileType::TypeScript => extract_typescript(path, false),
        FileType::Jsx | FileType::Tsx => extract_typescript(path, true),
        FileType::Unknown => extract_plaintext(path),
    }
}

#[cfg(test)]
mod tests {
    use super::extract;
    use crate::config::ExtractConfig;
    use crate::types::FileType;
    use anyhow::Result;
    use std::fs;

    #[test]
    fn extracts_mjs_via_javascript_path() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("sample.mjs");
        fs::write(
            &path,
            "import { helper } from './helper.mjs';\nexport function run() {}\n",
        )?;

        let content = extract(
            &path,
            FileType::from_extension("mjs"),
            &ExtractConfig::default(),
        )?;

        assert!(content.success);
        assert!(content
            .headings
            .contains(&"export function run".to_string()));
        assert!(content.links.contains(&"./helper.mjs".to_string()));

        Ok(())
    }

    #[test]
    fn extracts_cjs_via_javascript_path() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("sample.cjs");
        fs::write(&path, "class Worker {}\nmodule.exports = { Worker };\n")?;

        let content = extract(
            &path,
            FileType::from_extension("cjs"),
            &ExtractConfig::default(),
        )?;

        assert!(content.success);
        assert!(content.headings.contains(&"class Worker".to_string()));

        Ok(())
    }
}
