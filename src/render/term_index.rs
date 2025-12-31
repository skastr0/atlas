//! TERMS.md generation

use crate::types::FileFeatures;
use std::collections::HashMap;

/// Generate TERMS.md content
pub fn render_term_index(features: &[FileFeatures], top_n: usize) -> String {
    let mut output = String::new();

    output.push_str("# Term Index\n\n");
    output.push_str("_Mapping of key terms and phrases to files. Use this to find files about specific topics._\n\n");

    // Aggregate terms to files
    let mut term_files: HashMap<String, Vec<(&str, &str, f32)>> = HashMap::new();

    for file in features {
        for term in &file.top_terms {
            term_files
                .entry(term.term.clone())
                .or_default()
                .push((file.path.to_str().unwrap_or(""), &file.title, term.tfidf));
        }
    }

    // Sort terms by total TF-IDF across corpus
    let mut term_scores: Vec<_> = term_files
        .iter()
        .map(|(term, files)| {
            let total_score: f32 = files.iter().map(|(_, _, s)| s).sum();
            (term.clone(), total_score, files.len())
        })
        .collect();
    term_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top N terms
    for (term, _score, doc_count) in term_scores.into_iter().take(top_n) {
        output.push_str(&format!("## {}\n\n", term));
        output.push_str(&format!("_Found in {} files_\n\n", doc_count));

        if let Some(files) = term_files.get(&term) {
            // Sort files by TF-IDF for this term
            let mut sorted_files = files.clone();
            sorted_files.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

            for (path, title, _score) in sorted_files.iter().take(10) {
                output.push_str(&format!("- **{}** — `{}`\n", title, path));
            }
            output.push('\n');
        }
    }

    // Phrases section
    output.push_str("---\n\n");
    output.push_str("## Key Phrases\n\n");

    let mut phrase_files: HashMap<String, Vec<(&str, &str, f32)>> = HashMap::new();

    for file in features {
        for phrase in &file.top_phrases {
            phrase_files.entry(phrase.phrase.clone()).or_default().push((
                file.path.to_str().unwrap_or(""),
                &file.title,
                phrase.score,
            ));
        }
    }

    // Sort phrases by total score
    let mut phrase_scores: Vec<_> = phrase_files
        .iter()
        .map(|(phrase, files)| {
            let total_score: f32 = files.iter().map(|(_, _, s)| s).sum();
            (phrase.clone(), total_score, files.len())
        })
        .collect();
    phrase_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top phrases
    for (phrase, _score, doc_count) in phrase_scores.into_iter().take(top_n / 2) {
        output.push_str(&format!("### {}\n\n", phrase));
        output.push_str(&format!("_Found in {} files_\n\n", doc_count));

        if let Some(files) = phrase_files.get(&phrase) {
            let mut sorted_files = files.clone();
            sorted_files.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

            for (path, title, _score) in sorted_files.iter().take(5) {
                output.push_str(&format!("- **{}** — `{}`\n", title, path));
            }
            output.push('\n');
        }
    }

    output
}
