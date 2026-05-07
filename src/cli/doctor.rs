//! `atlas doctor` command - Report index health and drift

use crate::cache::{
    last_build_manifest_path, load_fingerprints, load_last_build_manifest, tantivy_backend,
};
use crate::config::Config;
use crate::extract::resolve_pdftotext;
use crate::types::{
    BuildFileIssue, DoctorCheck, DoctorCheckGroups, DoctorReport, DoctorSeverity,
    DoctorSeverityCounts, DoctorState, FileFeatures, LastBuildManifest,
    DOCTOR_RESULTS_CONTRACT_VERSION, LAST_BUILD_MANIFEST_VERSION,
};
use crate::{compare_fingerprints, LogLevel, Walker};
use anyhow::Result;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const ATLAS_DIR: &str = ".atlas";
const MAX_DETAIL_ITEMS: usize = 5;
const REQUIRED_ARTIFACTS: [&str; 6] = [
    "views/ROOT_ATLAS.md",
    "views/TERMS.md",
    "views/CONNECTIONS.md",
    "views/connections.mermaid",
    "views/connections.dot",
    "global/term_index.json",
];

pub fn run(root: &Path, json: bool, log_level: LogLevel) -> Result<()> {
    let report = inspect(root);

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else if log_level != LogLevel::Quiet {
        render_human_report(root, &report);
    }

    Ok(())
}

fn inspect(root: &Path) -> DoctorReport {
    let atlas_path = root.join(ATLAS_DIR);
    let mut checks = DoctorCheckGroups::default();

    let mut indexed_candidates = 0usize;
    let mut index_documents = 0usize;
    let mut changed_files = 0usize;
    let mut skipped_files = 0usize;
    let mut failed_files = 0usize;

    if !atlas_path.exists() {
        add_check(
            &mut checks,
            DoctorSeverity::Error,
            "initialization",
            "Initialization",
            "`.atlas` is missing; run `atlas init` first".to_string(),
            Vec::new(),
        );
        return finalize_report(
            checks,
            indexed_candidates,
            index_documents,
            changed_files,
            skipped_files,
            failed_files,
        );
    }

    add_check(
        &mut checks,
        DoctorSeverity::Info,
        "initialization",
        "Initialization",
        format!("Found `{}`", atlas_path.display()),
        Vec::new(),
    );

    let config = match Config::load_explicit(&atlas_path) {
        Ok(config) => {
            add_check(
                &mut checks,
                DoctorSeverity::Info,
                "config",
                "Config",
                "Loaded `.atlas/config.toml`".to_string(),
                Vec::new(),
            );
            Some(config)
        }
        Err(error) => {
            add_check(
                &mut checks,
                DoctorSeverity::Error,
                "config",
                "Config",
                "Failed to load `.atlas/config.toml`".to_string(),
                vec![error.to_string()],
            );
            None
        }
    };

    let current_files = match config.as_ref() {
        Some(config) => match walk_current_files(root, config) {
            Ok(files) => {
                indexed_candidates = files.len();
                Some(files)
            }
            Err(error) => {
                add_check(
                    &mut checks,
                    DoctorSeverity::Error,
                    "corpus_scan",
                    "Corpus scan",
                    "Failed to walk the configured corpus".to_string(),
                    vec![error.to_string()],
                );
                None
            }
        },
        None => None,
    };

    let fingerprints_path = atlas_path.join("fingerprints.jsonl");
    let cached_fingerprints = if !fingerprints_path.exists() {
        add_check(
            &mut checks,
            DoctorSeverity::Error,
            "fingerprints",
            "Fingerprints",
            "Saved fingerprints are missing; run `atlas build`".to_string(),
            vec![relative_atlas_path(&atlas_path, &fingerprints_path)],
        );
        None
    } else {
        match load_fingerprints(&fingerprints_path) {
            Ok(fingerprints) => {
                add_check(
                    &mut checks,
                    DoctorSeverity::Info,
                    "fingerprints",
                    "Fingerprints",
                    format!("Loaded {} saved fingerprint(s)", fingerprints.len()),
                    Vec::new(),
                );
                Some(fingerprints)
            }
            Err(error) => {
                add_check(
                    &mut checks,
                    DoctorSeverity::Error,
                    "fingerprints",
                    "Fingerprints",
                    "Saved fingerprints are unreadable".to_string(),
                    vec![error.to_string()],
                );
                None
            }
        }
    };

    let manifest_path = last_build_manifest_path(&atlas_path);
    let manifest = if !manifest_path.exists() {
        add_check(
            &mut checks,
            DoctorSeverity::Error,
            "last_build_manifest",
            "Last build manifest",
            "The last build manifest is missing; run `atlas build`".to_string(),
            vec![relative_atlas_path(&atlas_path, &manifest_path)],
        );
        None
    } else {
        match load_last_build_manifest(&manifest_path) {
            Ok(manifest) => {
                skipped_files = manifest.skipped.len();
                failed_files = manifest.failed.len();

                let (severity, summary, details) =
                    if manifest.version != LAST_BUILD_MANIFEST_VERSION {
                        (
                            DoctorSeverity::Error,
                            format!(
                                "Last build manifest version {} is unsupported",
                                manifest.version
                            ),
                            vec![format!("expected version {}", LAST_BUILD_MANIFEST_VERSION)],
                        )
                    } else if manifest.index_version != tantivy_backend::SEARCH_INDEX_VERSION {
                        (
                            DoctorSeverity::Error,
                            "Last build manifest targets a different index version".to_string(),
                            vec![format!(
                                "manifest={}, expected={}",
                                manifest.index_version,
                                tantivy_backend::SEARCH_INDEX_VERSION
                            )],
                        )
                    } else {
                        (
                            DoctorSeverity::Info,
                            format!(
                                "Loaded manifest for {} indexed candidate(s)",
                                manifest.indexed_candidates
                            ),
                            Vec::new(),
                        )
                    };

                add_check(
                    &mut checks,
                    severity,
                    "last_build_manifest",
                    "Last build manifest",
                    summary,
                    details,
                );
                Some(manifest)
            }
            Err(error) => {
                add_check(
                    &mut checks,
                    DoctorSeverity::Error,
                    "last_build_manifest",
                    "Last build manifest",
                    "The last build manifest is unreadable".to_string(),
                    vec![error.to_string()],
                );
                None
            }
        }
    };

    let indexed_features =
        match tantivy_backend::open_index(&tantivy_backend::index_dir(&atlas_path)) {
            Ok(index) => match tantivy_backend::load_all_features(&index) {
                Ok(features) => {
                    index_documents = features.len();
                    let (severity, summary, details) = match manifest.as_ref() {
                        Some(manifest) if manifest.indexed_documents != features.len() => (
                            DoctorSeverity::Error,
                            "Tantivy index document count does not match the last build manifest"
                                .to_string(),
                            vec![format!(
                                "manifest={}, index={}",
                                manifest.indexed_documents,
                                features.len()
                            )],
                        ),
                        _ => (
                            DoctorSeverity::Info,
                            format!(
                                "Loaded Tantivy index `{}` with {} document(s)",
                                tantivy_backend::SEARCH_INDEX_VERSION,
                                features.len()
                            ),
                            Vec::new(),
                        ),
                    };

                    add_check(
                        &mut checks,
                        severity,
                        "index",
                        "Tantivy index",
                        summary,
                        details,
                    );
                    Some(features)
                }
                Err(error) => {
                    add_check(
                        &mut checks,
                        DoctorSeverity::Error,
                        "index",
                        "Tantivy index",
                        "Tantivy index is unreadable".to_string(),
                        vec![error.to_string()],
                    );
                    None
                }
            },
            Err(error) => {
                add_check(
                    &mut checks,
                    DoctorSeverity::Error,
                    "index",
                    "Tantivy index",
                    "Tantivy index is missing or incompatible".to_string(),
                    vec![error.to_string()],
                );
                None
            }
        };

    let expected_documents = index_documents.max(
        manifest
            .as_ref()
            .map(|manifest| manifest.indexed_documents)
            .unwrap_or(0),
    );
    report_artifacts(&atlas_path, expected_documents, &mut checks);

    if let (Some(current_files), Some(cached_fingerprints)) =
        (current_files.as_ref(), cached_fingerprints.as_ref())
    {
        match compare_fingerprints(root, current_files, cached_fingerprints) {
            Ok(scan_result) => {
                changed_files = scan_result.new_files.len()
                    + scan_result.modified_files.len()
                    + scan_result.deleted_files.len();

                if changed_files == 0 {
                    add_check(
                        &mut checks,
                        DoctorSeverity::Info,
                        "corpus_delta",
                        "Corpus delta",
                        "Saved fingerprints match the current corpus".to_string(),
                        Vec::new(),
                    );
                } else {
                    add_check(
                        &mut checks,
                        DoctorSeverity::Warning,
                        "corpus_delta",
                        "Corpus delta",
                        format!(
                            "{} changed path(s) need a rebuild ({} new, {} modified, {} deleted)",
                            changed_files,
                            scan_result.new_files.len(),
                            scan_result.modified_files.len(),
                            scan_result.deleted_files.len()
                        ),
                        bound_details(delta_details(&scan_result)),
                    );
                }

                if let Some(indexed_features) = indexed_features.as_ref() {
                    report_index_drift(
                        &scan_result,
                        current_files,
                        indexed_features,
                        manifest.as_ref(),
                        &mut checks,
                    );
                }
            }
            Err(error) => {
                add_check(
                    &mut checks,
                    DoctorSeverity::Error,
                    "corpus_delta",
                    "Corpus delta",
                    "Failed to compare the current corpus against saved fingerprints".to_string(),
                    vec![error.to_string()],
                );
            }
        }
    }

    if let Some(manifest) = manifest.as_ref() {
        if manifest.skipped.is_empty() {
            add_check(
                &mut checks,
                DoctorSeverity::Info,
                "last_build_skips",
                "Last build skips",
                "Last build recorded no skipped files".to_string(),
                Vec::new(),
            );
        } else {
            add_check(
                &mut checks,
                DoctorSeverity::Warning,
                "last_build_skips",
                "Last build skips",
                format!("Last build skipped {} file(s)", manifest.skipped.len()),
                bound_details(issue_details(&manifest.skipped)),
            );
        }

        if manifest.failed.is_empty() {
            add_check(
                &mut checks,
                DoctorSeverity::Info,
                "last_build_failures",
                "Last build failures",
                "Last build recorded no failed files".to_string(),
                Vec::new(),
            );
        } else {
            add_check(
                &mut checks,
                DoctorSeverity::Error,
                "last_build_failures",
                "Last build failures",
                format!("Last build failed for {} file(s)", manifest.failed.len()),
                bound_details(issue_details(&manifest.failed)),
            );
        }
    }

    if pdf_dependency_relevant(
        config.as_ref(),
        current_files.as_deref(),
        indexed_features.as_deref(),
        manifest.as_ref(),
    ) {
        let custom_path = config
            .as_ref()
            .and_then(|config| config.extract.pdftotext_path.as_deref());

        match resolve_pdftotext(custom_path) {
            Ok(pdftotext) => add_check(
                &mut checks,
                DoctorSeverity::Info,
                "pdf_dependency",
                "PDF dependency",
                format!("`pdftotext` is available at `{pdftotext}`"),
                Vec::new(),
            ),
            Err(error) => {
                let mut details = pdf_signal_details(
                    config.as_ref(),
                    current_files.as_deref(),
                    indexed_features.as_deref(),
                    manifest.as_ref(),
                );
                details.push(error.to_string());
                add_check(
                    &mut checks,
                    DoctorSeverity::Error,
                    "pdf_dependency",
                    "PDF dependency",
                    "`pdftotext` is unavailable for configured or detected PDFs".to_string(),
                    bound_details(details),
                );
            }
        }
    }

    finalize_report(
        checks,
        indexed_candidates,
        index_documents,
        changed_files,
        skipped_files,
        failed_files,
    )
}

fn finalize_report(
    checks: DoctorCheckGroups,
    indexed_candidates: usize,
    index_documents: usize,
    changed_files: usize,
    skipped_files: usize,
    failed_files: usize,
) -> DoctorReport {
    let severity_counts = DoctorSeverityCounts {
        error: checks.error.len(),
        warning: checks.warning.len(),
        info: checks.info.len(),
    };
    let state = if severity_counts.error > 0 {
        DoctorState::Broken
    } else if severity_counts.warning > 0 {
        DoctorState::Stale
    } else {
        DoctorState::Clean
    };

    DoctorReport {
        version: DOCTOR_RESULTS_CONTRACT_VERSION,
        state,
        summary: crate::types::DoctorSummary {
            indexed_candidates,
            index_documents,
            requires_build: state != DoctorState::Clean,
            changed_files,
            skipped_files,
            failed_files,
            severity_counts,
        },
        checks,
    }
}

fn walk_current_files(root: &Path, config: &Config) -> Result<Vec<PathBuf>> {
    let walker = Walker::new(root, &config.scan);
    let mut files = walker.walk()?;
    files.sort_by_key(|path| normalize_path(path));
    Ok(files)
}

fn report_artifacts(atlas_path: &Path, expected_documents: usize, checks: &mut DoctorCheckGroups) {
    let mut missing = REQUIRED_ARTIFACTS
        .iter()
        .map(PathBuf::from)
        .filter(|relative| !atlas_path.join(relative).is_file())
        .map(|relative| normalize_path(&relative))
        .collect::<Vec<_>>();

    let folders_dir = atlas_path.join("views/folders");
    if !folders_dir.is_dir() {
        missing.push("views/folders".to_string());
    } else if expected_documents > 0 {
        let folder_index_count = fs::read_dir(&folders_dir)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.filter_map(Result::ok))
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("md"))
            .count();
        if folder_index_count == 0 {
            missing.push("views/folders/*.md".to_string());
        }
    }

    missing.sort();

    if missing.is_empty() {
        add_check(
            checks,
            DoctorSeverity::Info,
            "generated_artifacts",
            "Generated artifacts",
            "All generated artifacts are present".to_string(),
            Vec::new(),
        );
    } else {
        add_check(
            checks,
            DoctorSeverity::Error,
            "generated_artifacts",
            "Generated artifacts",
            format!("Missing {} generated artifact(s)", missing.len()),
            bound_details(missing),
        );
    }
}

fn report_index_drift(
    scan_result: &crate::ScanResult,
    current_files: &[PathBuf],
    indexed_features: &[FileFeatures],
    manifest: Option<&LastBuildManifest>,
    checks: &mut DoctorCheckGroups,
) {
    let current_paths = current_files
        .iter()
        .map(|path| normalize_path(path))
        .collect::<BTreeSet<_>>();
    let indexed_paths = indexed_features
        .iter()
        .map(|feature| normalize_path(&feature.path))
        .collect::<BTreeSet<_>>();
    let changed_paths = scan_result
        .new_files
        .iter()
        .chain(scan_result.modified_files.iter())
        .map(|path| normalize_path(path))
        .collect::<BTreeSet<_>>();
    let deleted_paths = scan_result
        .deleted_files
        .iter()
        .map(|path| normalize_path(path))
        .collect::<BTreeSet<_>>();

    let manifest_exclusions = manifest
        .map(|manifest| {
            manifest
                .skipped
                .iter()
                .chain(manifest.failed.iter())
                .map(|issue| normalize_str_path(&issue.path))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let missing_from_index = current_paths
        .difference(&indexed_paths)
        .filter(|path| !manifest_exclusions.contains(*path))
        .cloned()
        .collect::<Vec<_>>();
    let indexed_missing_on_disk = indexed_paths
        .difference(&current_paths)
        .cloned()
        .collect::<Vec<_>>();

    let mut warning_details = Vec::new();
    let mut error_details = Vec::new();

    for path in missing_from_index {
        if changed_paths.contains(&path) {
            warning_details.push(format!("missing from index: {path}"));
        } else {
            error_details.push(format!("missing from index: {path}"));
        }
    }

    for path in indexed_missing_on_disk {
        if deleted_paths.contains(&path) {
            warning_details.push(format!("indexed but missing on disk: {path}"));
        } else {
            error_details.push(format!("indexed but missing on disk: {path}"));
        }
    }

    warning_details.sort();
    error_details.sort();

    if !error_details.is_empty() {
        let mut details = error_details.clone();
        details.extend(warning_details.clone());
        add_check(
            checks,
            DoctorSeverity::Error,
            "index_drift",
            "Index drift",
            format!(
                "Detected {} unexplained index drift path(s)",
                error_details.len()
            ),
            bound_details(details),
        );
    } else if !warning_details.is_empty() {
        add_check(
            checks,
            DoctorSeverity::Warning,
            "index_drift",
            "Index drift",
            format!(
                "Detected {} drift path(s) explained by pending changes",
                warning_details.len()
            ),
            bound_details(warning_details),
        );
    } else {
        add_check(
            checks,
            DoctorSeverity::Info,
            "index_drift",
            "Index drift",
            "Index coverage matches the current corpus".to_string(),
            Vec::new(),
        );
    }
}

fn delta_details(scan_result: &crate::ScanResult) -> Vec<String> {
    let mut details = Vec::new();
    push_labeled_paths(&mut details, "new", &scan_result.new_files);
    push_labeled_paths(&mut details, "modified", &scan_result.modified_files);
    push_labeled_paths(&mut details, "deleted", &scan_result.deleted_files);
    details
}

fn push_labeled_paths(details: &mut Vec<String>, label: &str, paths: &[PathBuf]) {
    let mut normalized = paths
        .iter()
        .map(|path| normalize_path(path))
        .collect::<Vec<_>>();
    normalized.sort();
    details.extend(
        normalized
            .into_iter()
            .map(|path| format!("{label}: {path}")),
    );
}

fn issue_details(issues: &[BuildFileIssue]) -> Vec<String> {
    let mut details = issues
        .iter()
        .map(|issue| {
            let path = normalize_str_path(&issue.path);
            match issue.detail.as_deref() {
                Some(detail) => format!("{path} ({}, {detail})", issue.reason.label()),
                None => format!("{path} ({})", issue.reason.label()),
            }
        })
        .collect::<Vec<_>>();
    details.sort();
    details
}

fn pdf_dependency_relevant(
    config: Option<&Config>,
    current_files: Option<&[PathBuf]>,
    indexed_features: Option<&[FileFeatures]>,
    manifest: Option<&LastBuildManifest>,
) -> bool {
    config
        .and_then(|config| config.extract.pdftotext_path.as_ref())
        .is_some()
        || current_files
            .map(|files| files.iter().any(|path| is_pdf_path(path)))
            .unwrap_or(false)
        || indexed_features
            .map(|features| features.iter().any(|feature| is_pdf_path(&feature.path)))
            .unwrap_or(false)
        || manifest
            .map(|manifest| {
                manifest
                    .skipped
                    .iter()
                    .chain(manifest.failed.iter())
                    .any(|issue| is_pdf_str_path(&issue.path))
            })
            .unwrap_or(false)
}

fn pdf_signal_details(
    config: Option<&Config>,
    current_files: Option<&[PathBuf]>,
    indexed_features: Option<&[FileFeatures]>,
    manifest: Option<&LastBuildManifest>,
) -> Vec<String> {
    let mut details = Vec::new();

    if let Some(path) = config.and_then(|config| config.extract.pdftotext_path.as_deref()) {
        details.push(format!("configured pdftotext_path: {path}"));
    }

    let mut pdf_paths = BTreeSet::new();
    if let Some(current_files) = current_files {
        pdf_paths.extend(
            current_files
                .iter()
                .filter(|path| is_pdf_path(path))
                .map(|path| normalize_path(path)),
        );
    }
    if let Some(indexed_features) = indexed_features {
        pdf_paths.extend(
            indexed_features
                .iter()
                .filter(|feature| is_pdf_path(&feature.path))
                .map(|feature| normalize_path(&feature.path)),
        );
    }
    if let Some(manifest) = manifest {
        pdf_paths.extend(
            manifest
                .skipped
                .iter()
                .chain(manifest.failed.iter())
                .filter(|issue| is_pdf_str_path(&issue.path))
                .map(|issue| normalize_str_path(&issue.path)),
        );
    }

    details.extend(pdf_paths.into_iter().map(|path| format!("pdf: {path}")));
    details
}

fn add_check(
    checks: &mut DoctorCheckGroups,
    severity: DoctorSeverity,
    id: &str,
    label: &str,
    summary: String,
    details: Vec<String>,
) {
    let check = DoctorCheck {
        id: id.to_string(),
        label: label.to_string(),
        severity,
        summary,
        details,
    };

    match severity {
        DoctorSeverity::Error => checks.error.push(check),
        DoctorSeverity::Warning => checks.warning.push(check),
        DoctorSeverity::Info => checks.info.push(check),
    }
}

fn render_human_report(root: &Path, report: &DoctorReport) {
    println!("Doctor report for {}", root.display());
    println!(
        "State: {} | Indexed candidates: {} | Index docs: {} | Changed: {} | Skipped: {} | Failed: {} | Requires build: {}",
        report.state.label(),
        report.summary.indexed_candidates,
        report.summary.index_documents,
        report.summary.changed_files,
        report.summary.skipped_files,
        report.summary.failed_files,
        if report.summary.requires_build { "yes" } else { "no" }
    );

    render_group("Errors", &report.checks.error);
    render_group("Warnings", &report.checks.warning);
    render_group("Info", &report.checks.info);
}

fn render_group(label: &str, checks: &[DoctorCheck]) {
    if checks.is_empty() {
        return;
    }

    println!();
    println!("{} ({}):", label, checks.len());
    for check in checks {
        println!("- {}: {}", check.label, check.summary);
        for detail in &check.details {
            println!("  - {}", detail);
        }
    }
}

fn bound_details(details: Vec<String>) -> Vec<String> {
    if details.len() <= MAX_DETAIL_ITEMS {
        return details;
    }

    let remaining = details.len() - MAX_DETAIL_ITEMS;
    let mut bounded = details
        .into_iter()
        .take(MAX_DETAIL_ITEMS)
        .collect::<Vec<_>>();
    bounded.push(format!("... {remaining} more"));
    bounded
}

fn relative_atlas_path(atlas_path: &Path, path: &Path) -> String {
    path.strip_prefix(atlas_path)
        .map(normalize_path)
        .unwrap_or_else(|_| normalize_path(path))
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn normalize_str_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn is_pdf_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}

fn is_pdf_str_path(path: &str) -> bool {
    path.rsplit_once('.')
        .map(|(_, ext)| ext.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}
