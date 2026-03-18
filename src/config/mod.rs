//! Configuration types and defaults

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Root configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub scan: ScanConfig,
    pub extract: ExtractConfig,
    pub analyze: AnalyzeConfig,
    pub render: RenderConfig,
}

impl Config {
    pub fn load(cmap_dir: &Path) -> Result<Self> {
        let config_path = cmap_dir.join("config.toml");
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = match fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(_) => return Ok(Self::default()),
        };

        match toml::from_str(&content) {
            Ok(config) => Ok(config),
            Err(_) => Ok(Self::default()),
        }
    }
}

/// Scanning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScanConfig {
    /// Patterns to ignore (in addition to .gitignore)
    pub ignore: Vec<String>,
    /// File extensions to index
    pub include_extensions: Vec<String>,
}

pub const DEFAULT_INCLUDE_EXTENSIONS: &[&str] = &[
    // Prose
    "md", "txt", "pdf", "rst", "org", // Code
    "rs", "ts", "tsx", "js", "jsx", "mjs", "cjs", // Common config/text
    "json", "yml", "yaml", "toml", "sh", "sql",
];

fn default_include_extensions() -> Vec<String> {
    DEFAULT_INCLUDE_EXTENSIONS
        .iter()
        .map(|ext| (*ext).to_string())
        .collect()
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            ignore: vec![
                ".git".to_string(),
                ".cmap".to_string(),
                "node_modules".to_string(),
                "__pycache__".to_string(),
                "*.pyc".to_string(),
                ".DS_Store".to_string(),
            ],
            include_extensions: default_include_extensions(),
        }
    }
}

/// Text extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtractConfig {
    /// Max file size to process (bytes)
    pub max_file_size: usize,
    /// Snippet length (chars)
    pub snippet_length: usize,
    /// Path to pdftotext binary (auto-detected if not set)
    pub pdftotext_path: Option<String>,
}

impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10_000_000, // 10MB
            snippet_length: 400,
            pdftotext_path: None,
        }
    }
}

/// Analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AnalyzeConfig {
    /// Number of top terms per file
    pub top_terms: usize,
    /// Number of top phrases per file
    pub top_phrases: usize,
    /// Minimum term length
    pub min_term_length: usize,
    /// Maximum term length
    pub max_term_length: usize,
    /// Maximum digit ratio allowed in a term (0.0-1.0)
    pub max_digit_ratio: f32,
    /// Minimum document frequency for a term to be indexed
    pub min_df: usize,
    /// Maximum document frequency ratio (0.0-1.0)
    pub max_df_ratio: f32,
    /// Custom stopwords
    pub custom_stopwords: Vec<String>,
}

impl Default for AnalyzeConfig {
    fn default() -> Self {
        Self {
            top_terms: 20,
            top_phrases: 10,
            min_term_length: 3,
            max_term_length: 25,
            max_digit_ratio: 0.4,
            min_df: 2,
            max_df_ratio: 0.5,
            custom_stopwords: vec![],
        }
    }
}

/// Rendering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderConfig {
    /// Folder depth in ROOT_ATLAS.md
    pub atlas_folder_depth: usize,
    /// Max files to list per folder in atlas
    pub atlas_max_files_per_folder: usize,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            atlas_folder_depth: 3,
            atlas_max_files_per_folder: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, DEFAULT_INCLUDE_EXTENSIONS};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn load_missing_returns_default() {
        let dir = tempdir().expect("tempdir should work");
        let config = Config::load(dir.path()).expect("load should succeed");
        let default = Config::default();

        assert_eq!(config.scan.ignore, default.scan.ignore);
        assert_eq!(
            config.extract.snippet_length,
            default.extract.snippet_length
        );
    }

    #[test]
    fn load_malformed_returns_default() {
        let dir = tempdir().expect("tempdir should work");
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "[scan]\nignore = [\"oops\"").expect("write should succeed");

        let config = Config::load(dir.path()).expect("load should succeed");
        let default = Config::default();

        assert_eq!(config.analyze.top_terms, default.analyze.top_terms);
        assert_eq!(
            config.render.atlas_folder_depth,
            default.render.atlas_folder_depth
        );
    }

    #[test]
    fn load_valid_overrides_fields() {
        let dir = tempdir().expect("tempdir should work");
        let config_path = dir.path().join("config.toml");
        let content = r#"
[scan]
ignore = ["target"]
include_extensions = ["js", "json"]

[extract]
max_file_size = 1234
snippet_length = 321

[analyze]
top_terms = 7
top_phrases = 4
min_term_length = 5
custom_stopwords = ["alpha", "beta"]

[render]
atlas_folder_depth = 2
atlas_max_files_per_folder = 3
"#;
        fs::write(&config_path, content).expect("write should succeed");

        let config = Config::load(dir.path()).expect("load should succeed");

        assert_eq!(config.scan.ignore, vec!["target".to_string()]);
        assert_eq!(
            config.scan.include_extensions,
            vec!["js".to_string(), "json".to_string()]
        );
        assert_eq!(config.extract.snippet_length, 321);
        assert_eq!(config.analyze.top_terms, 7);
        assert_eq!(config.analyze.custom_stopwords, vec!["alpha", "beta"]);
        assert_eq!(config.render.atlas_max_files_per_folder, 3);
    }

    #[test]
    fn default_scan_extensions_cover_common_repo_files() {
        let config = Config::default();
        let expected: Vec<String> = DEFAULT_INCLUDE_EXTENSIONS
            .iter()
            .map(|ext| (*ext).to_string())
            .collect();

        assert_eq!(config.scan.include_extensions, expected);
    }
}
