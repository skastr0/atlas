use crate::types::LastBuildManifest;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub const LAST_BUILD_MANIFEST_FILE: &str = "last-build.json";

pub fn last_build_manifest_path(atlas_path: &Path) -> PathBuf {
    atlas_path.join(LAST_BUILD_MANIFEST_FILE)
}

pub fn load_last_build_manifest(path: &Path) -> Result<LastBuildManifest> {
    let bytes = fs::read(path)
        .with_context(|| format!("Failed to read last build manifest from {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Failed to parse last build manifest from {}",
            path.display()
        )
    })
}

pub fn save_last_build_manifest(path: &Path, manifest: &LastBuildManifest) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let payload =
        serde_json::to_vec_pretty(manifest).context("Failed to serialize last build manifest")?;
    fs::write(path, payload)
        .with_context(|| format!("Failed to write last build manifest to {}", path.display()))
}
