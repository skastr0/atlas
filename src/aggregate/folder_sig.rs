//! Folder signature computation

use crate::types::{FileFeatures, FolderSignature};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn rank_scored_strings(mut scored_items: Vec<(String, f32)>, top_n: usize) -> Vec<String> {
    scored_items.sort_by(|(left_value, left_score), (right_value, right_score)| {
        right_score
            .total_cmp(left_score)
            .then_with(|| left_value.cmp(right_value))
    });

    scored_items
        .into_iter()
        .take(top_n)
        .map(|(value, _)| value)
        .collect()
}

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
        let folder = file
            .path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();

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

        let top_terms = rank_scored_strings(terms.into_iter().collect(), top_n);
        let top_phrases = rank_scored_strings(phrases.into_iter().collect(), top_n);

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

#[cfg(test)]
mod tests {
    use super::{compute_folder_signatures, rank_scored_strings};
    use crate::types::{FileFeatures, FileType, PhraseScore, TermScore};
    use std::path::PathBuf;

    fn make_feature(
        id: &str,
        path: &str,
        top_terms: Vec<(&str, f32)>,
        top_phrases: Vec<(&str, f32)>,
    ) -> FileFeatures {
        FileFeatures {
            id: id.to_string(),
            path: PathBuf::from(path),
            file_type: FileType::Markdown,
            title: id.to_string(),
            snippet: String::new(),
            word_count: 0,
            char_count: 0,
            unique_term_count: top_terms.len(),
            top_terms: top_terms
                .into_iter()
                .map(|(term, tfidf)| TermScore {
                    term: term.to_string(),
                    tf: tfidf,
                    tfidf,
                })
                .collect(),
            top_phrases: top_phrases
                .into_iter()
                .map(|(phrase, score)| PhraseScore {
                    phrase: phrase.to_string(),
                    score,
                })
                .collect(),
            rake_phrases: Vec::new(),
            yake_keywords: Vec::new(),
            links_out: Vec::new(),
            headings: Vec::new(),
            extraction_ok: true,
            extracted_at: 0,
        }
    }

    #[test]
    fn rank_scored_strings_breaks_equal_scores_lexically() {
        let ranked = rank_scored_strings(
            vec![
                ("zeta".to_string(), 1.0),
                ("alpha".to_string(), 1.0),
                ("delta".to_string(), 0.5),
            ],
            2,
        );

        assert_eq!(ranked, vec!["alpha".to_string(), "zeta".to_string()]);
    }

    #[test]
    fn compute_folder_signatures_is_stable_for_reversed_file_order() {
        let first = make_feature(
            "doc-a",
            "docs/a.md",
            vec![("zeta", 1.0), ("alpha", 1.0)],
            vec![("z phrase", 2.0), ("a phrase", 2.0)],
        );
        let second = make_feature(
            "doc-b",
            "docs/b.md",
            vec![("alpha", 1.0), ("zeta", 1.0)],
            vec![("a phrase", 2.0), ("z phrase", 2.0)],
        );

        let forward = compute_folder_signatures(&[first.clone(), second.clone()], 1);
        let reversed = compute_folder_signatures(&[second, first], 1);
        let folder = PathBuf::from("docs");

        let forward_sig = forward.get(&folder).expect("missing docs signature");
        let reversed_sig = reversed.get(&folder).expect("missing docs signature");

        assert_eq!(forward_sig.top_terms, vec!["alpha".to_string()]);
        assert_eq!(forward_sig.top_phrases, vec!["a phrase".to_string()]);
        assert_eq!(forward_sig.top_terms, reversed_sig.top_terms);
        assert_eq!(forward_sig.top_phrases, reversed_sig.top_phrases);
    }
}
