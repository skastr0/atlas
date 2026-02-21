//! Global term index and document frequency

use crate::types::{FileFeatures, GlobalTermIndex, TermStats};
use std::collections::{BTreeMap, HashMap};

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
            terms: BTreeMap::new(),
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
    let mut terms = BTreeMap::new();
    for (term, docs) in term_docs {
        let df = docs.len();

        // Get top docs by TF
        let mut sorted_docs = docs;
        sorted_docs.sort_by(|(left_doc_id, left_tf), (right_doc_id, right_tf)| {
            right_tf
                .total_cmp(left_tf)
                .then_with(|| left_doc_id.cmp(right_doc_id))
        });

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
        file.top_terms.sort_by(|left, right| {
            right
                .tfidf
                .total_cmp(&left.tfidf)
                .then_with(|| left.term.cmp(&right.term))
        });
        file.top_terms.truncate(top_terms);
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_tfidf, build_term_index};
    use crate::types::{FileFeatures, FileType, GlobalTermIndex, TermScore, TermStats};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn make_file(id: &str, top_terms: Vec<(&str, f32)>) -> FileFeatures {
        FileFeatures {
            id: id.to_string(),
            path: PathBuf::from(format!("{id}.md")),
            file_type: FileType::Markdown,
            title: id.to_string(),
            snippet: String::new(),
            word_count: 0,
            char_count: 0,
            unique_term_count: top_terms.len(),
            top_terms: top_terms
                .into_iter()
                .map(|(term, tf)| TermScore {
                    term: term.to_string(),
                    tf,
                    tfidf: 0.0,
                })
                .collect(),
            top_phrases: Vec::new(),
            rake_phrases: Vec::new(),
            yake_keywords: Vec::new(),
            links_out: Vec::new(),
            headings: Vec::new(),
            extraction_ok: true,
            extracted_at: 0,
        }
    }

    #[test]
    fn build_term_index_sorts_top_docs_with_tie_breaker() {
        let forward = vec![
            make_file("doc-b", vec![("rust", 1.0)]),
            make_file("doc-a", vec![("rust", 1.0)]),
        ];
        let reversed = vec![
            make_file("doc-a", vec![("rust", 1.0)]),
            make_file("doc-b", vec![("rust", 1.0)]),
        ];

        let forward_index = build_term_index(&forward, 2, 1, 1.0);
        let reversed_index = build_term_index(&reversed, 2, 1, 1.0);

        let forward_docs = &forward_index
            .terms
            .get("rust")
            .expect("missing rust term")
            .top_docs;
        let reversed_docs = &reversed_index
            .terms
            .get("rust")
            .expect("missing rust term")
            .top_docs;

        assert_eq!(forward_docs, &vec!["doc-a".to_string(), "doc-b".to_string()]);
        assert_eq!(forward_docs, reversed_docs);
    }

    #[test]
    fn build_term_index_serialization_is_canonical_for_reversed_input() {
        let forward = vec![
            make_file("doc-b", vec![("beta", 1.0), ("alpha", 1.0)]),
            make_file("doc-a", vec![("alpha", 1.0), ("beta", 1.0)]),
        ];
        let reversed = vec![
            make_file("doc-a", vec![("alpha", 1.0), ("beta", 1.0)]),
            make_file("doc-b", vec![("beta", 1.0), ("alpha", 1.0)]),
        ];

        let forward_json =
            serde_json::to_string_pretty(&build_term_index(&forward, 2, 1, 1.0)).expect("serialize");
        let reversed_json = serde_json::to_string_pretty(&build_term_index(&reversed, 2, 1, 1.0))
            .expect("serialize");

        let alpha_pos = forward_json.find("\"alpha\"").expect("alpha missing");
        let beta_pos = forward_json.find("\"beta\"").expect("beta missing");

        assert!(alpha_pos < beta_pos, "expected alpha before beta: {forward_json}");
        assert_eq!(forward_json, reversed_json);
    }

    #[test]
    fn apply_tfidf_sorts_equal_scores_lexically_before_truncation() {
        let mut features = vec![make_file("doc-1", vec![("beta", 2.0), ("alpha", 2.0)])];
        let index = GlobalTermIndex {
            total_docs: 2,
            terms: BTreeMap::from([
                (
                    "alpha".to_string(),
                    TermStats {
                        df: 1,
                        top_docs: vec!["doc-1".to_string()],
                    },
                ),
                (
                    "beta".to_string(),
                    TermStats {
                        df: 1,
                        top_docs: vec!["doc-1".to_string()],
                    },
                ),
            ]),
        };

        apply_tfidf(&mut features, &index, 1);

        assert_eq!(features[0].top_terms.len(), 1);
        assert_eq!(features[0].top_terms[0].term, "alpha");
    }
}
