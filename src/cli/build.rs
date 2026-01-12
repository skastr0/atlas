//! `cmap build` command - Build index and generate views

use crate::config::Config;
use crate::LogLevel;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Import our modules
use crate::aggregate::{apply_tfidf, build_term_index, compute_folder_signatures};
use crate::analyze::compute_features;
use crate::cache::{load_all_features, load_fingerprints, save_features, save_fingerprints};
use crate::extract::extract;
use crate::render::{
    render_all_folder_indexes, render_atlas, render_connections, render_term_index,
};
use crate::scan::{compare_fingerprints, compute_content_hash, Walker};
use crate::types::FileType;

const CMAP_DIR: &str = ".cmap";

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
    let files = walker.walk()?;

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

    // Step 3: Determine which files to process
    let files_to_process: Vec<_> = if force {
        files.clone()
    } else {
        scan_result
            .new_files
            .iter()
            .chain(scan_result.modified_files.iter())
            .cloned()
            .collect()
    };

    // Step 4: Load existing features (for files we're not reprocessing)
    let features_cache_dir = cmap_path.join("cache/features");
    let mut all_features = if force {
        Vec::new()
    } else {
        load_all_features(&features_cache_dir).unwrap_or_default()
    };

    // Remove features for files we're reprocessing or deleted
    let reprocess_set: std::collections::HashSet<_> = files_to_process.iter().collect();
    let deleted_set: std::collections::HashSet<_> = scan_result.deleted_files.iter().collect();
    all_features.retain(|f| !reprocess_set.contains(&f.path) && !deleted_set.contains(&f.path));

    if !files_to_process.is_empty() {
        if log_level != LogLevel::Quiet {
            println!("Processing {} files...", files_to_process.len());
        }

        // Step 5: Extract and analyze files (in parallel)
        let new_features: Vec<_> = files_to_process
            .par_iter()
            .filter_map(|rel_path| {
                let full_path = root.join(rel_path);

                // Determine file type
                let ext = rel_path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();
                let file_type = FileType::from_extension(&ext);

                let metadata = match fs::metadata(&full_path) {
                    Ok(metadata) => metadata,
                    Err(e) => {
                        if log_level == LogLevel::Debug {
                            eprintln!("  Error reading metadata {}: {}", rel_path.display(), e);
                        }
                        return None;
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
                    return None;
                }

                // Extract text
                let content = match extract(&full_path, file_type, &config.extract) {
                    Ok(c) => c,
                    Err(e) => {
                        if log_level == LogLevel::Debug {
                            eprintln!("  Error extracting {}: {}", rel_path.display(), e);
                        }
                        return None;
                    }
                };

                // Compute content hash for ID
                let id = compute_content_hash(&full_path)
                    .unwrap_or_else(|_| format!("{:x}", md5_path(rel_path)));

                // Compute features
                let features = compute_features(
                    &id,
                    rel_path.clone(),
                    file_type,
                    &content,
                    &config.analyze,
                    &config.extract,
                );

                Some(features)
            })
            .collect();

        // Save new features to cache
        for features in &new_features {
            if let Err(e) = save_features(&features_cache_dir, features) {
                if log_level == LogLevel::Debug {
                    eprintln!("  Error caching {}: {}", features.path.display(), e);
                }
            }
        }

        all_features.extend(new_features);
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
        let folder_slug = folder
            .to_string_lossy()
            .replace('/', "_")
            .replace('\\', "_");
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
