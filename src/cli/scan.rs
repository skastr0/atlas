//! `cmap scan` command - Read-only delta preview over the configured corpus

use crate::cache::load_fingerprints;
use crate::config::Config;
use crate::types::{
    ScanDeltaGroup, ScanDeltaGroups, ScanDeltaReport, ScanDeltaSummary,
    SCAN_RESULTS_CONTRACT_VERSION,
};
use crate::{compare_fingerprints, LogLevel, Walker};
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const CMAP_DIR: &str = ".cmap";
const MAX_RENDERED_PATHS: usize = 5;

pub fn run(root: &Path, _dry_run: bool, json: bool, log_level: LogLevel) -> Result<()> {
    let cmap_path = root.join(CMAP_DIR);
    if !cmap_path.exists() {
        anyhow::bail!("Not initialized. Run `cmap init` first.");
    }

    let config = load_scan_config(&cmap_path)?;
    let walker = Walker::new(root, &config.scan);
    let mut files = walker.walk().context("Failed to walk configured corpus")?;
    files.sort_by_cached_key(|path| normalize_path(path));

    let fingerprints_path = cmap_path.join("fingerprints.jsonl");
    let cached_fingerprints = load_fingerprints(&fingerprints_path).with_context(|| {
        format!(
            "Failed to load saved fingerprints from {}",
            fingerprints_path.display()
        )
    })?;
    let scan_result = compare_fingerprints(root, &files, &cached_fingerprints)
        .context("Failed to compare the current corpus against saved fingerprints")?;
    let report = build_report(&files, &scan_result);

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else if log_level != LogLevel::Quiet {
        render_human_report(root, &report);
    }

    Ok(())
}

fn load_scan_config(cmap_path: &Path) -> Result<Config> {
    Config::load_explicit(cmap_path).with_context(|| {
        let config_path = cmap_path.join("config.toml");
        format!("Failed to load scan config from {}", config_path.display())
    })
}

fn build_report(current_files: &[PathBuf], scan_result: &crate::ScanResult) -> ScanDeltaReport {
    let new_files = sorted_paths(&scan_result.new_files);
    let modified_files = sorted_paths(&scan_result.modified_files);
    let deleted_files = sorted_paths(&scan_result.deleted_files);
    let changed_current: BTreeSet<String> = new_files
        .iter()
        .chain(modified_files.iter())
        .cloned()
        .collect();
    let unchanged_files: Vec<String> = current_files
        .iter()
        .map(|path| normalize_path(path))
        .filter(|path| !changed_current.contains(path))
        .collect();
    let changed_files = new_files.len() + modified_files.len() + deleted_files.len();

    ScanDeltaReport {
        version: SCAN_RESULTS_CONTRACT_VERSION,
        read_only: true,
        indexed_candidates: scan_result.total_files,
        summary: ScanDeltaSummary {
            changed_files,
            new_files: new_files.len(),
            modified_files: modified_files.len(),
            deleted_files: deleted_files.len(),
            unchanged_files: unchanged_files.len(),
            requires_build: changed_files > 0,
        },
        groups: ScanDeltaGroups {
            new_files: ScanDeltaGroup {
                count: new_files.len(),
                paths: new_files,
            },
            modified_files: ScanDeltaGroup {
                count: modified_files.len(),
                paths: modified_files,
            },
            deleted_files: ScanDeltaGroup {
                count: deleted_files.len(),
                paths: deleted_files,
            },
            unchanged_files: ScanDeltaGroup {
                count: unchanged_files.len(),
                paths: unchanged_files,
            },
        },
    }
}

fn sorted_paths(paths: &[PathBuf]) -> Vec<String> {
    let mut normalized: Vec<String> = paths.iter().map(|path| normalize_path(path)).collect();
    normalized.sort();
    normalized
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn render_human_report(root: &Path, report: &ScanDeltaReport) {
    println!("Scan delta for {}", root.display());
    println!("Indexed candidates: {}", report.indexed_candidates);

    if report.summary.requires_build {
        println!(
            "Build impact: {} changed path(s) would be reprocessed by the next build",
            report.summary.changed_files
        );
    } else {
        println!("Build impact: no changes; the next build can reuse current fingerprints");
        println!("Representative changes: none");
    }

    println!(
        "New: {} | Modified: {} | Deleted: {} | Unchanged: {}",
        report.summary.new_files,
        report.summary.modified_files,
        report.summary.deleted_files,
        report.summary.unchanged_files
    );

    render_group("New files", &report.groups.new_files);
    render_group("Modified files", &report.groups.modified_files);
    render_group("Deleted files", &report.groups.deleted_files);
}

fn render_group(label: &str, group: &ScanDeltaGroup) {
    if group.paths.is_empty() {
        return;
    }

    println!();
    println!("{} ({}):", label, group.count);
    for path in group.paths.iter().take(MAX_RENDERED_PATHS) {
        println!("  - {}", path);
    }

    if group.count > MAX_RENDERED_PATHS {
        println!("  ... {} more", group.count - MAX_RENDERED_PATHS);
    }
}

#[cfg(test)]
mod tests {
    use super::build_report;
    use crate::{Fingerprint, ScanResult};
    use std::path::PathBuf;

    #[test]
    fn report_orders_groups_deterministically() {
        let current_files = vec![PathBuf::from("b.md"), PathBuf::from("a.md")];
        let scan_result = ScanResult {
            fingerprints: vec![Fingerprint {
                path: PathBuf::from("b.md"),
                mtime: 0,
                size: 0,
                content_hash: None,
            }],
            new_files: vec![PathBuf::from("b.md")],
            modified_files: vec![PathBuf::from("a.md")],
            deleted_files: vec![PathBuf::from("z.md"), PathBuf::from("c.md")],
            total_files: current_files.len(),
        };

        let report = build_report(&current_files, &scan_result);

        assert_eq!(report.groups.new_files.paths, vec!["b.md"]);
        assert_eq!(report.groups.modified_files.paths, vec!["a.md"]);
        assert_eq!(report.groups.deleted_files.paths, vec!["c.md", "z.md"]);
        assert!(report.groups.unchanged_files.paths.is_empty());
        assert!(report.summary.requires_build);
    }
}
