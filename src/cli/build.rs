//! `cmap build` command - Build index and generate views

use crate::config::Config;
use crate::LogLevel;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Import our modules
use crate::aggregate::{apply_tfidf, build_term_index, compute_folder_signatures};
use crate::analyze::compute_features;
use crate::cache::{
    last_build_manifest_path, load_fingerprints, save_fingerprints, save_last_build_manifest,
    tantivy_backend,
};
use crate::extract::extract;
use crate::render::{
    render_all_folder_indexes, render_atlas, render_connections, render_term_index,
};
use crate::scan::{compare_fingerprints, compute_content_hash, Walker};
use crate::types::{
    BuildFileIssue, BuildFileIssueReason, FileFeatures, FileType, LastBuildManifest,
    LAST_BUILD_MANIFEST_VERSION,
};

const CMAP_DIR: &str = ".cmap";

enum ProcessedFileOutcome {
    Indexed(Box<(FileFeatures, String)>),
    Skipped(BuildFileIssue),
    Failed(BuildFileIssue),
}

pub fn run(root: &Path, _changed_only: bool, force: bool, log_level: LogLevel) -> Result<()> {
    let cmap_path = root.join(CMAP_DIR);

    // Check if initialized
    if !cmap_path.exists() {
        anyhow::bail!("Not initialized. Run `cmap init` first.");
    }

    // Load config
    let config_path = cmap_path.join("config.toml");
    let config = Config::load(&cmap_path)?;

    if log_level != LogLevel::Quiet {
        if config_path.exists() {
            println!("Using config: {}", config_path.display());
        } else {
            println!("Using default config");
        }
        println!("Scanning {}...", root.display());
    }

    // Step 1: Scan for files
    let walker = Walker::new(root, &config.scan);
    let mut files = walker.walk()?;
    files.sort();

    if log_level != LogLevel::Quiet {
        println!("Found {} files", files.len());
    }

    // Step 2: Compare with cached fingerprints
    let fingerprints_path = cmap_path.join("fingerprints.jsonl");
    let cached_fingerprints = if force {
        HashMap::new()
    } else {
        load_fingerprints(&fingerprints_path)?
    };

    let scan_result = compare_fingerprints(root, &files, &cached_fingerprints)?;

    if log_level != LogLevel::Quiet {
        println!(
            "  {} new, {} modified, {} deleted, {} unchanged",
            scan_result.new_files.len(),
            scan_result.modified_files.len(),
            scan_result.deleted_files.len(),
            scan_result.total_files
                - scan_result.new_files.len()
                - scan_result.modified_files.len()
        );
    }

    // Step 4: Load existing features and initialize Tantivy
    let index_dir = tantivy_backend::index_dir(&cmap_path);

    if force && index_dir.exists() {
        std::fs::remove_dir_all(&index_dir)
            .context("Failed to clear index directory for force build")?;
    }

    let prepared_index = tantivy_backend::prepare_index(&index_dir)?;
    let tantivy_index = prepared_index.index;
    let needs_full_reindex = force || prepared_index.needs_reindex;

    // Step 3: Determine which files to process
    let mut files_to_process: Vec<_> = if needs_full_reindex {
        files.clone()
    } else {
        scan_result
            .new_files
            .iter()
            .chain(scan_result.modified_files.iter())
            .cloned()
            .collect()
    };
    files_to_process.sort();

    let mut all_features = if needs_full_reindex {
        Vec::new()
    } else {
        tantivy_backend::load_all_features(&tantivy_index).unwrap_or_default()
    };
    let mut build_skips = Vec::new();
    let mut build_failures = Vec::new();

    // Remove features for files we're reprocessing or deleted
    let reprocess_set: std::collections::HashSet<_> = files_to_process.iter().collect();
    let deleted_set: std::collections::HashSet<_> = scan_result.deleted_files.iter().collect();

    let mut ids_to_delete = Vec::new();
    for f in &all_features {
        if reprocess_set.contains(&f.path) || deleted_set.contains(&f.path) {
            ids_to_delete.push(f.id.clone());
        }
    }

    all_features.retain(|f| !reprocess_set.contains(&f.path) && !deleted_set.contains(&f.path));

    let mut index_writer = tantivy_index
        .writer(50_000_000)
        .context("Failed to create tantivy index writer")?;

    if !ids_to_delete.is_empty() {
        if let Err(e) =
            tantivy_backend::delete_documents(&tantivy_index, &mut index_writer, &ids_to_delete)
        {
            if log_level == LogLevel::Debug {
                eprintln!("  Error deleting old features: {}", e);
            }
        }
    }

    if !files_to_process.is_empty() {
        if log_level != LogLevel::Quiet {
            println!("Processing {} files...", files_to_process.len());
        }

        // Step 5: Extract and analyze files (in parallel)
        let processed_files: Vec<_> = files_to_process
            .par_iter()
            .map(|rel_path| process_file(root, rel_path, &config, log_level))
            .collect();

        let mut new_features = Vec::new();
        let mut skipped_files = Vec::new();
        let mut failed_files = Vec::new();

        for processed in processed_files {
            match processed {
                ProcessedFileOutcome::Indexed(features) => new_features.push(*features),
                ProcessedFileOutcome::Skipped(issue) => skipped_files.push(issue),
                ProcessedFileOutcome::Failed(issue) => failed_files.push(issue),
            }
        }

        skipped_files.sort_by(|a, b| a.path.cmp(&b.path));
        failed_files.sort_by(|a, b| a.path.cmp(&b.path));

        // Save new features to cache
        if let Err(e) =
            tantivy_backend::add_documents(&tantivy_index, &mut index_writer, &new_features)
        {
            if log_level == LogLevel::Debug {
                eprintln!("  Error caching features to Tantivy: {}", e);
            }
        }

        all_features.extend(new_features.into_iter().map(|(f, _)| f));

        build_skips = skipped_files;
        build_failures = failed_files;
    }

    // Commit any changes to the index
    if !ids_to_delete.is_empty() || !files_to_process.is_empty() {
        if let Err(e) = index_writer.commit() {
            if log_level == LogLevel::Debug {
                eprintln!("  Error committing tantivy index: {}", e);
            }
        }
    }

    if log_level != LogLevel::Quiet {
        println!("Building index from {} files...", all_features.len());
    }

    // Step 6: Build global term index and compute TF-IDF
    let term_index = build_term_index(
        &all_features,
        5,
        config.analyze.min_df,
        config.analyze.max_df_ratio,
    );
    apply_tfidf(&mut all_features, &term_index, config.analyze.top_terms);

    // Step 7: Compute folder signatures
    let folder_sigs = compute_folder_signatures(&all_features, 20);

    // Step 8: Render views
    if log_level != LogLevel::Quiet {
        println!("Generating views...");
    }

    let views_dir = cmap_path.join("views");
    fs::create_dir_all(&views_dir)?;

    // ROOT_ATLAS.md
    let atlas = render_atlas(&all_features, &folder_sigs, &config.render);
    fs::write(views_dir.join("ROOT_ATLAS.md"), atlas)?;

    // TERMS.md
    let terms = render_term_index(&all_features, 100);
    fs::write(views_dir.join("TERMS.md"), terms)?;

    // CONNECTIONS.md + diagram outputs
    let connections = render_connections(&all_features);
    fs::write(views_dir.join("CONNECTIONS.md"), connections.markdown)?;
    fs::write(views_dir.join("connections.mermaid"), connections.mermaid)?;
    fs::write(views_dir.join("connections.dot"), connections.dot)?;

    // Per-folder INDEX.md files
    let folders_dir = views_dir.join("folders");
    fs::create_dir_all(&folders_dir)?;

    let folder_indexes = render_all_folder_indexes(&all_features, &folder_sigs, &config.render);
    for (folder, content) in folder_indexes {
        let folder_slug = folder.to_string_lossy().replace(['/', '\\'], "_");
        let folder_slug = if folder_slug.is_empty() {
            "root".to_string()
        } else {
            folder_slug
        };
        fs::write(folders_dir.join(format!("{}.md", folder_slug)), content)?;
    }

    // Step 9: Save fingerprints
    save_fingerprints(&fingerprints_path, &scan_result.fingerprints)?;

    // Step 10: Save global stats
    let global_dir = cmap_path.join("global");
    fs::create_dir_all(&global_dir)?;
    let term_index_json = serde_json::to_string_pretty(&term_index)?;
    fs::write(global_dir.join("term_index.json"), term_index_json)?;

    let manifest = LastBuildManifest {
        version: LAST_BUILD_MANIFEST_VERSION,
        index_version: tantivy_backend::SEARCH_INDEX_VERSION.to_string(),
        indexed_candidates: scan_result.total_files,
        indexed_documents: all_features.len(),
        processed_files: files_to_process.len(),
        full_reindex: needs_full_reindex,
        skipped: build_skips,
        failed: build_failures,
    };
    let manifest_path = last_build_manifest_path(&cmap_path);
    save_last_build_manifest(&manifest_path, &manifest)?;

    if log_level != LogLevel::Quiet {
        println!("✓ Done!");
        println!("  Views written to {}", views_dir.display());
        println!("  - ROOT_ATLAS.md");
        println!("  - TERMS.md");
        println!("  - CONNECTIONS.md");
        println!("  - connections.mermaid");
        println!("  - connections.dot");
        println!("  - folders/*.md ({} folders)", folder_sigs.len());
    }

    Ok(())
}

// Simple hash for path-based ID fallback
fn md5_path(path: &Path) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

fn process_file(
    root: &Path,
    rel_path: &Path,
    config: &Config,
    log_level: LogLevel,
) -> ProcessedFileOutcome {
    let full_path = root.join(rel_path);
    let ext = rel_path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    let file_type = FileType::from_extension(&ext);

    let metadata = match fs::metadata(&full_path) {
        Ok(metadata) => metadata,
        Err(error) => {
            if log_level == LogLevel::Debug {
                eprintln!("  Error reading metadata {}: {}", rel_path.display(), error);
            }
            return ProcessedFileOutcome::Failed(build_issue(
                rel_path,
                BuildFileIssueReason::MetadataUnreadable,
                None,
            ));
        }
    };

    if metadata.len() > config.extract.max_file_size as u64 {
        if log_level == LogLevel::Debug {
            eprintln!(
                "  Skipping {} ({} bytes > max {})",
                rel_path.display(),
                metadata.len(),
                config.extract.max_file_size
            );
        }
        return ProcessedFileOutcome::Skipped(build_issue(
            rel_path,
            BuildFileIssueReason::FileTooLarge,
            Some(format!(
                "{} bytes > max {}",
                metadata.len(),
                config.extract.max_file_size
            )),
        ));
    }

    let content = match extract(&full_path, file_type, &config.extract) {
        Ok(content) => content,
        Err(error) => {
            if log_level == LogLevel::Debug {
                eprintln!("  Error extracting {}: {}", rel_path.display(), error);
            }
            return ProcessedFileOutcome::Failed(build_issue(
                rel_path,
                classify_extraction_failure(file_type, &error),
                extraction_detail(file_type, &error),
            ));
        }
    };

    let id =
        compute_content_hash(&full_path).unwrap_or_else(|_| format!("{:x}", md5_path(rel_path)));
    let features = compute_features(
        &id,
        rel_path.to_path_buf(),
        file_type,
        &content,
        &config.analyze,
        &config.extract,
    );

    ProcessedFileOutcome::Indexed(Box::new((features, content.text)))
}

fn build_issue(
    path: &Path,
    reason: BuildFileIssueReason,
    detail: Option<String>,
) -> BuildFileIssue {
    BuildFileIssue {
        path: path.to_string_lossy().replace('\\', "/"),
        reason,
        detail,
    }
}

fn classify_extraction_failure(file_type: FileType, error: &anyhow::Error) -> BuildFileIssueReason {
    if file_type == FileType::Pdf && error_mentions_pdftotext(error) {
        BuildFileIssueReason::PdftotextUnavailable
    } else {
        BuildFileIssueReason::ExtractionFailed
    }
}

fn extraction_detail(file_type: FileType, error: &anyhow::Error) -> Option<String> {
    if file_type == FileType::Pdf && error_mentions_pdftotext(error) {
        Some("pdftotext is unavailable for PDF extraction".to_string())
    } else {
        None
    }
}

fn error_mentions_pdftotext(error: &anyhow::Error) -> bool {
    error
        .chain()
        .any(|cause| cause.to_string().contains("pdftotext"))
}
