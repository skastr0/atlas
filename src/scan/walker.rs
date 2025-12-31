//! Directory traversal with ignore patterns

use crate::config::ScanConfig;
use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Walk a directory tree, respecting ignore patterns
pub struct Walker<'a> {
    root: &'a Path,
    config: &'a ScanConfig,
}

impl<'a> Walker<'a> {
    pub fn new(root: &'a Path, config: &'a ScanConfig) -> Self {
        Self { root, config }
    }

    /// Get all files matching the configuration
    pub fn walk(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        let mut builder = WalkBuilder::new(self.root);
        builder
            .hidden(true) // Skip hidden files by default
            .git_ignore(true) // Respect .gitignore
            .git_global(true)
            .git_exclude(true);

        // Add custom ignore patterns
        for pattern in &self.config.ignore {
            builder.add_ignore(self.root.join(pattern));
        }

        for entry in builder.build() {
            let entry = entry?;
            let path = entry.path();

            // Skip directories
            if !path.is_file() {
                continue;
            }

            // Check extension
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if self.config.include_extensions.contains(&ext_str) {
                    // Store relative path
                    if let Ok(relative) = path.strip_prefix(self.root) {
                        files.push(relative.to_path_buf());
                    } else {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }

        Ok(files)
    }
}
