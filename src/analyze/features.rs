//! Per-file feature computation

use super::{
    extract_bigrams, extract_links, extract_trigrams, filter_stopwords, tokenize_with_config,
    top_ngrams, RakeExtractor, YakeExtractor,
};
use crate::config::{AnalyzeConfig, ExtractConfig};
use crate::extract::ExtractedContent;
use crate::types::{FileFeatures, FileType, PhraseScore, TermScore};
use std::collections::HashMap;
use std::path::PathBuf;

/// Compute features for a file
pub fn compute_features(
    id: &str,
    path: PathBuf,
    file_type: FileType,
    content: &ExtractedContent,
    analyze_config: &AnalyzeConfig,
    extract_config: &ExtractConfig,
) -> FileFeatures {
    // Tokenize
    let tokens = tokenize_with_config(&content.text, analyze_config);
    let filtered_tokens = filter_stopwords(tokens.clone(), &analyze_config.custom_stopwords);

    // Compute term frequencies
    let mut term_counts: HashMap<String, usize> = HashMap::new();
    for token in &filtered_tokens {
        *term_counts.entry(token.clone()).or_insert(0) += 1;
    }

    let unique_term_count = term_counts.len();

    // Get top terms by raw frequency (TF-IDF will be computed in global pass)
    let total_tokens = filtered_tokens.len() as f32;
    let mut top_terms: Vec<_> = term_counts
        .iter()
        .map(|(term, &count)| TermScore {
            term: term.clone(),
            tf: count as f32 / total_tokens.max(1.0),
            tfidf: 0.0, // Will be filled in global pass
        })
        .collect();
    top_terms.sort_by(|a, b| b.tf.partial_cmp(&a.tf).unwrap_or(std::cmp::Ordering::Equal));
    // Keep a larger candidate set so IDF can promote rare terms.
    let initial_term_limit = analyze_config.top_terms.max(200);
    top_terms.truncate(initial_term_limit);

    // Extract phrases (bigrams + trigrams)
    let bigrams = extract_bigrams(&filtered_tokens);
    let trigrams = extract_trigrams(&filtered_tokens);

    // Merge and get top phrases
    let mut all_phrases: HashMap<String, usize> = bigrams;
    for (phrase, count) in trigrams {
        *all_phrases.entry(phrase).or_insert(0) += count;
    }

    let top_phrase_tuples = top_ngrams(all_phrases, analyze_config.top_phrases);
    let top_phrases: Vec<_> = top_phrase_tuples
        .into_iter()
        .map(|(phrase, count)| PhraseScore {
            phrase,
            score: count as f32,
        })
        .collect();

    // Extract RAKE phrases
    let rake_extractor = RakeExtractor::new(analyze_config.top_phrases);
    let rake_phrases = rake_extractor.extract(&content.text, analyze_config);

    // Extract YAKE keywords
    let yake_keywords = YakeExtractor::default().extract(&content.text, analyze_config);

    // Extract links
    let links = extract_links(&content.text);

    // Compute snippet (first N chars or first paragraph)
    let snippet = extract_snippet(&content.text, extract_config.snippet_length);

    // Get title (from content extraction or derive from path)
    let title = content
        .title
        .clone()
        .unwrap_or_else(|| derive_title_from_path(&path));

    // Get current timestamp
    let extracted_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    FileFeatures {
        id: id.to_string(),
        path,
        file_type,
        title,
        snippet,
        word_count: tokens.len(),
        char_count: content.text.len(),
        unique_term_count,
        top_terms,
        top_phrases,
        rake_phrases,
        yake_keywords,
        links_out: links,
        headings: content.headings.clone(),
        extraction_ok: content.success,
        extracted_at,
    }
}

fn extract_snippet(text: &str, max_chars: usize) -> String {
    // Try to get first paragraph
    let first_para = text
        .split("\n\n")
        .find(|p| !p.trim().is_empty() && p.trim().len() > 20);

    if let Some(para) = first_para {
        if para.len() <= max_chars {
            return para.trim().to_string();
        }
    }

    // Fall back to first N chars
    let snippet: String = text.chars().take(max_chars).collect();

    // Try to break at word boundary
    if let Some(last_space) = snippet.rfind(' ') {
        format!("{}...", &snippet[..last_space])
    } else {
        format!("{}...", snippet)
    }
}

fn derive_title_from_path(path: &PathBuf) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string())
}
