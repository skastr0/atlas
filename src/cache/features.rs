//! Feature cache management

use crate::types::FileFeatures;
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// Load cached features for a file
pub fn load_features(cache_dir: &Path, id: &str) -> Result<Option<FileFeatures>> {
    let path = cache_dir.join(format!("{}.json", id));

    if !path.exists() {
        return Ok(None);
    }

    let file = File::open(&path).context("Failed to open feature cache file")?;
    let reader = BufReader::new(file);
    let features: FileFeatures =
        serde_json::from_reader(reader).context("Failed to parse feature cache")?;

    Ok(Some(features))
}

/// Save features to cache
pub fn save_features(cache_dir: &Path, features: &FileFeatures) -> Result<()> {
    fs::create_dir_all(cache_dir).context("Failed to create cache directory")?;

    let path = cache_dir.join(format!("{}.json", features.id));
    let file = File::create(&path).context("Failed to create feature cache file")?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, features).context("Failed to write feature cache")?;

    Ok(())
}

/// Load all cached features
pub fn load_all_features(cache_dir: &Path) -> Result<Vec<FileFeatures>> {
    let mut features = Vec::new();

    if !cache_dir.exists() {
        return Ok(features);
    }

    for entry in fs::read_dir(cache_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            if let Ok(f) = serde_json::from_reader::<_, FileFeatures>(reader) {
                features.push(f);
            }
        }
    }

    Ok(features)
}

/// Remove stale cache entries
pub fn remove_stale_features(cache_dir: &Path, valid_ids: &[String]) -> Result<usize> {
    let valid_set: std::collections::HashSet<_> = valid_ids.iter().collect();
    let mut removed = 0;

    if !cache_dir.exists() {
        return Ok(0);
    }

    for entry in fs::read_dir(cache_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Some(stem) = path.file_stem() {
                let id = stem.to_string_lossy().to_string();
                if !valid_set.contains(&id) {
                    fs::remove_file(&path)?;
                    removed += 1;
                }
            }
        }
    }

    Ok(removed)
}
