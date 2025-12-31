//! Folder signature computation

use crate::types::{FileFeatures, FolderSignature};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Compute signatures for all folders
pub fn compute_folder_signatures(
    features: &[FileFeatures],
    top_n: usize,
) -> HashMap<PathBuf, FolderSignature> {
    let mut folder_terms: HashMap<PathBuf, HashMap<String, f32>> = HashMap::new();
    let mut folder_phrases: HashMap<PathBuf, HashMap<String, f32>> = HashMap::new();
    let mut folder_counts: HashMap<PathBuf, usize> = HashMap::new();

    // Aggregate terms and phrases by folder
    for file in features {
        let folder = file.path.parent().map(Path::to_path_buf).unwrap_or_default();

        // Aggregate for this folder and all parent folders
        let mut current = Some(folder.as_path());
        while let Some(f) = current {
            let folder_path = f.to_path_buf();

            *folder_counts.entry(folder_path.clone()).or_insert(0) += 1;

            // Aggregate terms
            let terms = folder_terms.entry(folder_path.clone()).or_default();
            for term_score in &file.top_terms {
                *terms.entry(term_score.term.clone()).or_insert(0.0) += term_score.tfidf;
            }

            // Aggregate phrases
            let phrases = folder_phrases.entry(folder_path.clone()).or_default();
            for phrase_score in &file.top_phrases {
                *phrases.entry(phrase_score.phrase.clone()).or_insert(0.0) += phrase_score.score;
            }

            current = f.parent();
        }
    }

    // Build signatures
    let mut signatures = HashMap::new();

    for (folder, terms) in folder_terms {
        let count = folder_counts.get(&folder).copied().unwrap_or(0);
        let phrases = folder_phrases.remove(&folder).unwrap_or_default();

        // Get top terms
        let mut sorted_terms: Vec<_> = terms.into_iter().collect();
        sorted_terms.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_terms: Vec<String> = sorted_terms.into_iter().take(top_n).map(|(t, _)| t).collect();

        // Get top phrases
        let mut sorted_phrases: Vec<_> = phrases.into_iter().collect();
        sorted_phrases.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_phrases: Vec<String> = sorted_phrases
            .into_iter()
            .take(top_n)
            .map(|(p, _)| p)
            .collect();

        signatures.insert(
            folder.clone(),
            FolderSignature {
                path: folder,
                file_count: count,
                top_terms,
                top_phrases,
            },
        );
    }

    signatures
}
