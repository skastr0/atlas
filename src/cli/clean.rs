//! `cmap clean` command - Remove cached data

use crate::cache::tantivy_backend;
use crate::LogLevel;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const CMAP_DIR: &str = ".cmap";

pub fn run(root: &Path, all: bool, log_level: LogLevel) -> Result<()> {
    let cmap_path = root.join(CMAP_DIR);

    if !cmap_path.exists() {
        if log_level != LogLevel::Quiet {
            println!("No .cmap directory found at {}", root.display());
        }
        return Ok(());
    }

    // Clean cache directories
    let cache_path = cmap_path.join("cache");
    if cache_path.exists() {
        fs::remove_dir_all(&cache_path).context("Failed to remove cache directory")?;
        fs::create_dir_all(cache_path.join("text"))?;
        if log_level != LogLevel::Quiet {
            println!("✓ Cleared cache");
        }
    }

    // Clean index
    let index_path = tantivy_backend::index_dir(&cmap_path);
    if index_path.exists() {
        fs::remove_dir_all(&index_path).context("Failed to remove tantivy index directory")?;
        fs::create_dir_all(&index_path)?;
        if log_level != LogLevel::Quiet {
            println!("✓ Cleared tantivy index");
        }
    }

    // Clean fingerprints
    let fingerprints_path = cmap_path.join("fingerprints.jsonl");
    if fingerprints_path.exists() {
        fs::remove_file(&fingerprints_path).context("Failed to remove fingerprints")?;
        if log_level != LogLevel::Quiet {
            println!("✓ Cleared fingerprints");
        }
    }

    // Clean global stats
    let global_path = cmap_path.join("global");
    if global_path.exists() {
        fs::remove_dir_all(&global_path).context("Failed to remove global directory")?;
        fs::create_dir_all(&global_path)?;
        if log_level != LogLevel::Quiet {
            println!("✓ Cleared global stats");
        }
    }

    // Optionally clean views
    if all {
        let views_path = cmap_path.join("views");
        if views_path.exists() {
            fs::remove_dir_all(&views_path).context("Failed to remove views directory")?;
            fs::create_dir_all(views_path.join("folders"))?;
            if log_level != LogLevel::Quiet {
                println!("✓ Cleared views");
            }
        }
    }

    if log_level != LogLevel::Quiet {
        println!("Done!");
    }

    Ok(())
}
