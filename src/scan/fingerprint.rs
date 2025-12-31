//! File fingerprinting for change detection

use crate::types::{ChangeStatus, Fingerprint, ScanResult};
use anyhow::Result;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Compute fingerprint for a single file
pub fn compute_fingerprint(root: &Path, relative_path: &Path) -> Result<Fingerprint> {
    let full_path = root.join(relative_path);
    let metadata = fs::metadata(&full_path)?;

    let mtime = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Ok(Fingerprint {
        path: relative_path.to_path_buf(),
        mtime,
        size: metadata.len(),
        content_hash: None,
    })
}

/// Compute content hash (Blake3) for a file
pub fn compute_content_hash(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    let hash = blake3::hash(&data);
    Ok(hash.to_hex().to_string())
}

/// Load fingerprints from JSONL file
pub fn load_fingerprints(path: &Path) -> Result<HashMap<PathBuf, Fingerprint>> {
    let mut fingerprints = HashMap::new();

    if !path.exists() {
        return Ok(fingerprints);
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let fp: Fingerprint = serde_json::from_str(&line)?;
        fingerprints.insert(fp.path.clone(), fp);
    }

    Ok(fingerprints)
}

/// Save fingerprints to JSONL file
pub fn save_fingerprints(path: &Path, fingerprints: &[Fingerprint]) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for fp in fingerprints {
        let line = serde_json::to_string(fp)?;
        writeln!(writer, "{}", line)?;
    }

    writer.flush()?;
    Ok(())
}

/// Compare current files against cached fingerprints
pub fn compare_fingerprints(
    root: &Path,
    current_files: &[PathBuf],
    cached: &HashMap<PathBuf, Fingerprint>,
) -> Result<ScanResult> {
    let mut result = ScanResult {
        fingerprints: Vec::with_capacity(current_files.len()),
        new_files: Vec::new(),
        modified_files: Vec::new(),
        deleted_files: Vec::new(),
        total_files: current_files.len(),
    };

    // Check current files against cache
    for path in current_files {
        let fp = compute_fingerprint(root, path)?;

        let status = match cached.get(path) {
            None => ChangeStatus::New,
            Some(cached_fp) => {
                if fp.mtime != cached_fp.mtime || fp.size != cached_fp.size {
                    ChangeStatus::Modified
                } else {
                    ChangeStatus::Unchanged
                }
            }
        };

        match status {
            ChangeStatus::New => result.new_files.push(path.clone()),
            ChangeStatus::Modified => result.modified_files.push(path.clone()),
            _ => {}
        }

        result.fingerprints.push(fp);
    }

    // Find deleted files
    let current_set: std::collections::HashSet<_> = current_files.iter().collect();
    for cached_path in cached.keys() {
        if !current_set.contains(cached_path) {
            result.deleted_files.push(cached_path.clone());
        }
    }

    Ok(result)
}
