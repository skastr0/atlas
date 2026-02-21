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
            term_files.entry(term.term.clone()).or_default().push((
                file.path.to_str().unwrap_or(""),
                &file.title,
                term.tfidf,
            ));
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
    term_scores.sort_by(
        |(left_term, left_score, _), (right_term, right_score, _)| {
            right_score
                .total_cmp(left_score)
                .then_with(|| left_term.cmp(right_term))
        },
    );

    // Take top N terms
    for (term, _score, doc_count) in term_scores.into_iter().take(top_n) {
        output.push_str(&format!("## {}\n\n", term));
        output.push_str(&format!("_Found in {} files_\n\n", doc_count));

        if let Some(files) = term_files.get(&term) {
            // Sort files by TF-IDF for this term
            let mut sorted_files = files.clone();
            sorted_files.sort_by(
                |(left_path, left_title, left_score), (right_path, right_title, right_score)| {
                    right_score
                        .total_cmp(left_score)
                        .then_with(|| left_path.cmp(right_path))
                        .then_with(|| left_title.cmp(right_title))
                },
            );

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
            phrase_files
                .entry(phrase.phrase.clone())
                .or_default()
                .push((file.path.to_str().unwrap_or(""), &file.title, phrase.score));
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
    phrase_scores.sort_by(
        |(left_phrase, left_score, _), (right_phrase, right_score, _)| {
            right_score
                .total_cmp(left_score)
                .then_with(|| left_phrase.cmp(right_phrase))
        },
    );

    // Take top phrases
    for (phrase, _score, doc_count) in phrase_scores.into_iter().take(top_n / 2) {
        output.push_str(&format!("### {}\n\n", phrase));
        output.push_str(&format!("_Found in {} files_\n\n", doc_count));

        if let Some(files) = phrase_files.get(&phrase) {
            let mut sorted_files = files.clone();
            sorted_files.sort_by(
                |(left_path, left_title, left_score), (right_path, right_title, right_score)| {
                    right_score
                        .total_cmp(left_score)
                        .then_with(|| left_path.cmp(right_path))
                        .then_with(|| left_title.cmp(right_title))
                },
            );

            for (path, title, _score) in sorted_files.iter().take(5) {
                output.push_str(&format!("- **{}** — `{}`\n", title, path));
            }
            output.push('\n');
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::render_term_index;
    use crate::types::{FileFeatures, FileType, PhraseScore, TermScore};
    use std::path::PathBuf;

    fn make_feature(
        id: &str,
        path: &str,
        title: &str,
        top_terms: Vec<(&str, f32)>,
        top_phrases: Vec<(&str, f32)>,
    ) -> FileFeatures {
        FileFeatures {
            id: id.to_string(),
            path: PathBuf::from(path),
            file_type: FileType::Markdown,
            title: title.to_string(),
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

    fn assert_order(content: &str, first: &str, second: &str) {
        let first_idx = content.find(first).expect("first marker missing");
        let second_idx = content.find(second).expect("second marker missing");

        assert!(
            first_idx < second_idx,
            "expected `{first}` before `{second}` in:\n{content}"
        );
    }

    fn between<'a>(content: &'a str, start: &str, end: &str) -> &'a str {
        let start_idx = content.find(start).expect("start marker missing");
        let start_slice = &content[start_idx..];
        let end_idx = start_slice.find(end).expect("end marker missing");
        &start_slice[..end_idx]
    }

    #[test]
    fn renders_deterministic_term_and_phrase_order_for_ties() {
        let first = make_feature(
            "doc-z",
            "docs/z.md",
            "Z title",
            vec![("zeta", 1.0), ("alpha", 1.0)],
            vec![("z phrase", 2.0), ("a phrase", 2.0)],
        );
        let second = make_feature(
            "doc-a",
            "docs/a.md",
            "A title",
            vec![("alpha", 1.0), ("zeta", 1.0)],
            vec![("a phrase", 2.0), ("z phrase", 2.0)],
        );

        let forward = render_term_index(&[first.clone(), second.clone()], 4);
        let reversed = render_term_index(&[second, first], 4);

        assert_eq!(forward, reversed);
        assert_order(&forward, "## alpha\n\n", "## zeta\n\n");

        let alpha_section = between(&forward, "## alpha\n\n", "## zeta\n\n");
        assert_order(
            alpha_section,
            "- **A title** — `docs/a.md`",
            "- **Z title** — `docs/z.md`",
        );

        assert_order(&forward, "### a phrase\n\n", "### z phrase\n\n");

        let phrase_section = between(&forward, "### a phrase\n\n", "### z phrase\n\n");
        assert_order(
            phrase_section,
            "- **A title** — `docs/a.md`",
            "- **Z title** — `docs/z.md`",
        );
    }

    #[test]
    fn sorts_term_files_by_title_when_score_and_path_tie() {
        let first = make_feature(
            "doc-z",
            "docs/same.md",
            "Zulu",
            vec![("alpha", 1.0)],
            vec![],
        );
        let second = make_feature(
            "doc-a",
            "docs/same.md",
            "Alpha",
            vec![("alpha", 1.0)],
            vec![],
        );

        let rendered = render_term_index(&[first, second], 2);
        let alpha_section = between(&rendered, "## alpha\n\n", "---\n\n");

        assert_order(
            alpha_section,
            "- **Alpha** — `docs/same.md`",
            "- **Zulu** — `docs/same.md`",
        );
    }
}
