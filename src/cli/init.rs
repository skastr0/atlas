//! `cmap init` command - Initialize .cmap directory

use crate::config::Config;
use crate::LogLevel;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const CMAP_DIR: &str = ".cmap";

pub fn run(root: &Path, log_level: LogLevel) -> Result<()> {
    let cmap_path = root.join(CMAP_DIR);

    if cmap_path.exists() {
        if log_level != LogLevel::Quiet {
            println!("✓ .cmap already exists at {}", cmap_path.display());
        }
        return Ok(());
    }

    // Create directory structure
    fs::create_dir_all(cmap_path.join("cache/text"))
        .context("Failed to create cache/text directory")?;
    fs::create_dir_all(cmap_path.join("cache/features"))
        .context("Failed to create cache/features directory")?;
    fs::create_dir_all(cmap_path.join("global")).context("Failed to create global directory")?;
    fs::create_dir_all(cmap_path.join("views/folders"))
        .context("Failed to create views/folders directory")?;

    // Write default config
    let config = Config::default();
    let config_path = cmap_path.join("config.toml");
    let config_toml = toml_string(&config);
    fs::write(&config_path, config_toml).context("Failed to write config.toml")?;

    if log_level != LogLevel::Quiet {
        println!("✓ Initialized .cmap at {}", cmap_path.display());
        println!("  Edit .cmap/config.toml to customize settings");
    }

    Ok(())
}

fn toml_string(config: &Config) -> String {
    // Manual TOML generation to keep comments and ordering stable
    let mut s = String::new();

    s.push_str("[scan]\n");
    s.push_str("# Patterns to ignore (in addition to .gitignore)\n");
    s.push_str("ignore = [\n");
    for pattern in &config.scan.ignore {
        s.push_str(&format!("    \"{}\",\n", pattern));
    }
    s.push_str("]\n\n");

    s.push_str("# File extensions to index\n");
    s.push_str("include_extensions = [\n");
    for ext in &config.scan.include_extensions {
        s.push_str(&format!("    \"{}\",\n", ext));
    }
    s.push_str("]\n\n");

    s.push_str("[extract]\n");
    s.push_str(&format!(
        "# Max file size to process (bytes)\nmax_file_size = {}\n\n",
        config.extract.max_file_size
    ));
    s.push_str(&format!(
        "# Snippet length (chars)\nsnippet_length = {}\n\n",
        config.extract.snippet_length
    ));
    s.push_str("# Path to pdftotext binary (auto-detected if not set)\n");
    s.push_str("# pdftotext_path = \"/usr/bin/pdftotext\"\n\n");

    s.push_str("[analyze]\n");
    s.push_str(&format!(
        "# Number of top terms per file\ntop_terms = {}\n\n",
        config.analyze.top_terms
    ));
    s.push_str(&format!(
        "# Number of top phrases per file\ntop_phrases = {}\n\n",
        config.analyze.top_phrases
    ));
    s.push_str(&format!(
        "# Minimum term length\nmin_term_length = {}\n\n",
        config.analyze.min_term_length
    ));
    s.push_str(&format!(
        "# Maximum term length\nmax_term_length = {}\n\n",
        config.analyze.max_term_length
    ));
    s.push_str(&format!(
        "# Maximum digit ratio in a term (0.0-1.0)\nmax_digit_ratio = {}\n\n",
        config.analyze.max_digit_ratio
    ));
    s.push_str(&format!(
        "# Minimum document frequency for a term\nmin_df = {}\n\n",
        config.analyze.min_df
    ));
    s.push_str(&format!(
        "# Maximum document frequency ratio (0.0-1.0)\nmax_df_ratio = {}\n\n",
        config.analyze.max_df_ratio
    ));

    s.push_str("[render]\n");
    s.push_str(&format!(
        "# Folder depth in ROOT_ATLAS.md\natlas_folder_depth = {}\n\n",
        config.render.atlas_folder_depth
    ));
    s.push_str(&format!(
        "# Max files to list per folder in atlas\natlas_max_files_per_folder = {}\n",
        config.render.atlas_max_files_per_folder
    ));

    s
}
