//! Global term index and document frequency

use crate::types::{FileFeatures, GlobalTermIndex, TermStats};
use std::collections::HashMap;

/// Build global term index from file features
pub fn build_term_index(
    features: &[FileFeatures],
    top_docs_per_term: usize,
    min_df: usize,
    max_df_ratio: f32,
) -> GlobalTermIndex {
    let total_docs = features.len();
    let mut term_docs: HashMap<String, Vec<(String, f32)>> = HashMap::new();

    // Collect all terms with their document and TF score
    for file in features {
        for term_score in &file.top_terms {
            term_docs
                .entry(term_score.term.clone())
                .or_default()
                .push((file.id.clone(), term_score.tf));
        }
    }

    if total_docs == 0 {
        return GlobalTermIndex {
            total_docs,
            terms: HashMap::new(),
        };
    }

    // Drop terms outside document frequency thresholds
    term_docs.retain(|_, docs| {
        let df = docs.len();
        if df < min_df {
            return false;
        }
        let df_ratio = df as f32 / total_docs as f32;
        df_ratio <= max_df_ratio
    });

    // Build the index
    let mut terms = HashMap::new();
    for (term, docs) in term_docs {
        let df = docs.len();

        // Get top docs by TF
        let mut sorted_docs = docs;
        sorted_docs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_docs: Vec<String> = sorted_docs
            .into_iter()
            .take(top_docs_per_term)
            .map(|(id, _)| id)
            .collect();

        terms.insert(term, TermStats { df, top_docs });
    }

    GlobalTermIndex { total_docs, terms }
}

/// Update file features with TF-IDF scores using global index
pub fn apply_tfidf(features: &mut [FileFeatures], index: &GlobalTermIndex, top_terms: usize) {
    let total_docs = index.total_docs as f32;

    for file in features {
        for term_score in &mut file.top_terms {
            if let Some(stats) = index.terms.get(&term_score.term) {
                let idf = (total_docs / stats.df as f32).ln() + 1.0;
                term_score.tfidf = term_score.tf * idf;
            }
        }

        // Re-sort by TF-IDF
        file.top_terms
            .sort_by(|a, b| b.tfidf.partial_cmp(&a.tfidf).unwrap_or(std::cmp::Ordering::Equal));
        file.top_terms.truncate(top_terms);
    }
}
