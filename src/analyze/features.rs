//! Per-file feature computation

use super::{
    extract_bigrams, extract_links, extract_trigrams, filter_stopwords, tokenize_with_config,
    top_ngrams, RakeExtractor, YakeExtractor,
};
use crate::config::{AnalyzeConfig, ExtractConfig};
use crate::extract::ExtractedContent;
use crate::types::{FileFeatures, FileType, Link, LinkType, PhraseScore, TermScore};
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
    let mut links = extract_links(&content.text);
    if file_type.is_code() && !content.links.is_empty() {
        links.extend(extract_code_links(file_type, &content.links));
    }

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

fn extract_code_links(file_type: FileType, links: &[String]) -> Vec<Link> {
    let mut output = Vec::new();

    for target in links {
        match file_type {
            FileType::Rust => {
                for normalized in normalize_rust_use_targets(target) {
                    output.push(Link {
                        target: normalized,
                        link_type: LinkType::Internal,
                    });
                }
            }
            FileType::TypeScript | FileType::Tsx => {
                if let Some(normalized) = normalize_typescript_import(target) {
                    output.push(Link {
                        target: normalized,
                        link_type: LinkType::Internal,
                    });
                }
            }
            _ => {}
        }
    }

    output
}

fn normalize_typescript_import(target: &str) -> Option<String> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with('.') || trimmed.starts_with('/') {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn normalize_rust_use_targets(target: &str) -> Vec<String> {
    let Some(cleaned) = strip_rust_use_target(target) else {
        return Vec::new();
    };

    let Some((prefix, rest)) = rust_prefix_and_rest(&cleaned) else {
        return Vec::new();
    };

    let segments: Vec<&str> = rest.split("::").filter(|seg| !seg.is_empty()).collect();
    if segments.is_empty() {
        return Vec::new();
    }

    let mut targets = Vec::new();
    let full = format!("{}{}", prefix, segments.join("/"));
    targets.push(full.clone());

    if segments.len() > 1 {
        let trimmed = format!("{}{}", prefix, segments[..segments.len() - 1].join("/"));
        if trimmed != full {
            targets.push(trimmed);
        }
    }

    targets
}

fn strip_rust_use_target(target: &str) -> Option<String> {
    let mut cleaned = target.trim();
    if cleaned.is_empty() {
        return None;
    }

    if let Some((left, _)) = cleaned.split_once('{') {
        cleaned = left.trim_end_matches("::").trim();
    }

    if let Some((left, _)) = cleaned.split_once(" as ") {
        cleaned = left.trim();
    }

    let cleaned = cleaned.trim_end_matches(';').trim();
    let cleaned = cleaned.trim_end_matches("::*").trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn rust_prefix_and_rest(target: &str) -> Option<(String, String)> {
    if let Some(rest) = target.strip_prefix("crate::") {
        return Some(("/src/".to_string(), rest.to_string()));
    }

    if let Some(rest) = target.strip_prefix("self::") {
        return Some((String::new(), rest.to_string()));
    }

    if target.starts_with("super::") {
        let mut rest = target;
        let mut depth = 0usize;
        while let Some(stripped) = rest.strip_prefix("super::") {
            depth += 1;
            rest = stripped;
        }
        if rest.is_empty() {
            return None;
        }
        let prefix = "../".repeat(depth.saturating_sub(1));
        return Some((prefix, rest.to_string()));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{normalize_rust_use_targets, normalize_typescript_import};

    #[test]
    fn normalizes_typescript_relative_imports() {
        assert_eq!(normalize_typescript_import("./foo"), Some("./foo".to_string()));
        assert_eq!(normalize_typescript_import("../bar"), Some("../bar".to_string()));
        assert_eq!(normalize_typescript_import("react"), None);
    }

    #[test]
    fn normalizes_rust_use_targets() {
        let targets = normalize_rust_use_targets("crate::config::{AnalyzeConfig, ExtractConfig}");
        assert_eq!(targets, vec!["/src/config".to_string()]);

        let targets = normalize_rust_use_targets("crate::extract::rust::extract_rust");
        assert_eq!(
            targets,
            vec![
                "/src/extract/rust/extract_rust".to_string(),
                "/src/extract/rust".to_string(),
            ]
        );

        let targets = normalize_rust_use_targets("super::treesitter::parse");
        assert_eq!(
            targets,
            vec!["treesitter/parse".to_string(), "treesitter".to_string()]
        );
    }
}
