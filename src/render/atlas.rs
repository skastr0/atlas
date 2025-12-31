//! ROOT_ATLAS.md generation

use crate::config::RenderConfig;
use crate::types::{FileFeatures, FolderSignature};
use std::collections::HashMap;
use std::path::PathBuf;

/// Generate ROOT_ATLAS.md content
pub fn render_atlas(
    features: &[FileFeatures],
    folder_sigs: &HashMap<PathBuf, FolderSignature>,
    config: &RenderConfig,
) -> String {
    let mut output = String::new();

    output.push_str("# Knowledge Base Atlas\n\n");
    output.push_str("_Auto-generated map of this knowledge base. Use this to understand what exists before searching._\n\n");

    // Summary stats
    output.push_str("## Overview\n\n");
    output.push_str(&format!("- **Total files:** {}\n", features.len()));

    let total_words: usize = features.iter().map(|f| f.word_count).sum();
    output.push_str(&format!("- **Total words:** {}\n", total_words));

    let folders: std::collections::HashSet<_> = features
        .iter()
        .filter_map(|f| f.path.parent())
        .collect();
    output.push_str(&format!("- **Folders:** {}\n\n", folders.len()));

    // Folder tree with signatures
    output.push_str("## Folder Structure\n\n");
    let tree = build_folder_tree(features, folder_sigs, config.atlas_folder_depth);
    output.push_str(&tree);
    output.push('\n');

    // Objective slices
    output.push_str(&render_objective_slices(features));

    // Global top terms
    output.push_str("## Top Concepts\n\n");
    let global_terms = aggregate_global_terms(features, 30);
    for (term, _score) in global_terms {
        output.push_str(&format!("- {}\n", term));
    }
    output.push('\n');

    // Navigation hints
    output.push_str("## Navigation\n\n");
    output.push_str("- Each folder has an `INDEX.md` with detailed file listings\n");
    output.push_str("- See `TERMS.md` for concept-to-file mappings\n");
    output.push_str("- File paths are relative to the knowledge base root\n");

    output
}

fn render_objective_slices(features: &[FileFeatures]) -> String {
    let mut output = String::new();

    output.push_str("## Objective Slices\n\n");

    let mut by_size: Vec<&FileFeatures> = features.iter().collect();
    by_size.sort_by_key(|f| std::cmp::Reverse(f.word_count));
    output.push_str("### Largest Files\n\n");
    for file in by_size.into_iter().take(10) {
        push_slice_entry(&mut output, file, &file.word_count.to_string());
    }
    output.push('\n');

    let mut by_links: Vec<&FileFeatures> = features.iter().collect();
    by_links.sort_by_key(|f| std::cmp::Reverse(f.links_out.len()));
    output.push_str("### Most Connected\n\n");
    for file in by_links.into_iter().take(10) {
        push_slice_entry(&mut output, file, &file.links_out.len().to_string());
    }
    output.push('\n');

    let mut by_distinctive: Vec<(&FileFeatures, f32)> = features
        .iter()
        .map(|f| (f, max_term_tfidf(f)))
        .collect();
    by_distinctive.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });
    output.push_str("### Most Distinctive\n\n");
    for (file, score) in by_distinctive.into_iter().take(10) {
        push_slice_entry(&mut output, file, &format!("{:.3}", score));
    }
    output.push('\n');

    let mut by_diverse: Vec<&FileFeatures> = features.iter().collect();
    by_diverse.sort_by_key(|f| std::cmp::Reverse(f.unique_term_count));
    output.push_str("### Most Diverse\n\n");
    for file in by_diverse.into_iter().take(10) {
        push_slice_entry(&mut output, file, &file.unique_term_count.to_string());
    }
    output.push('\n');

    output
}

fn push_slice_entry(output: &mut String, file: &FileFeatures, metric: &str) {
    output.push_str(&format!(
        "- **{}** ({}) - {}\n",
        file.title,
        metric,
        file.path.display()
    ));
}

fn max_term_tfidf(file: &FileFeatures) -> f32 {
    file.top_terms
        .iter()
        .map(|term| term.tfidf)
        .fold(0.0, f32::max)
}

fn build_folder_tree(
    features: &[FileFeatures],
    folder_sigs: &HashMap<PathBuf, FolderSignature>,
    max_depth: usize,
) -> String {
    let mut output = String::new();

    // Get unique folders
    let mut folders: Vec<_> = features
        .iter()
        .filter_map(|f| f.path.parent())
        .map(|p| p.to_path_buf())
        .collect();
    folders.sort();
    folders.dedup();

    // Filter by depth
    let folders: Vec<_> = folders
        .into_iter()
        .filter(|p| p.components().count() <= max_depth)
        .collect();

    for folder in folders {
        let depth = folder.components().count();
        let indent = "  ".repeat(depth);

        let sig = folder_sigs.get(&folder);
        let file_count = sig.map(|s| s.file_count).unwrap_or(0);
        let top_phrases = sig
            .map(|s| s.top_phrases.iter().take(3).cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        let folder_name = folder
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "(root)".to_string());

        if top_phrases.is_empty() {
            output.push_str(&format!("{}- **{}/** ({} files)\n", indent, folder_name, file_count));
        } else {
            output.push_str(&format!(
                "{}- **{}/** ({} files) — {}\n",
                indent,
                folder_name,
                file_count,
                top_phrases.join(", ")
            ));
        }
    }

    output
}

fn aggregate_global_terms(features: &[FileFeatures], top_n: usize) -> Vec<(String, f32)> {
    let mut term_scores: HashMap<String, f32> = HashMap::new();

    for file in features {
        for term in &file.top_terms {
            *term_scores.entry(term.term.clone()).or_insert(0.0) += term.tfidf;
        }
    }

    let mut sorted: Vec<_> = term_scores.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sorted.truncate(top_n);
    sorted
}

