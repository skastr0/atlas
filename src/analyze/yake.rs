//! YAKE keyword extraction

use super::tokenize::is_stopword;
use crate::config::AnalyzeConfig;
use crate::types::KeywordScore;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use unicode_segmentation::UnicodeSegmentation;

const DEFAULT_MAX_KEYWORDS: usize = 20;
const EPSILON: f32 = 1e-6;

#[derive(Debug, Clone)]
pub struct YakeExtractor {
    max_keywords: usize,
}

impl YakeExtractor {
    pub fn new(max_keywords: usize) -> Self {
        Self { max_keywords }
    }

    pub fn extract(&self, text: &str, config: &AnalyzeConfig) -> Vec<KeywordScore> {
        let sentences = split_sentences(text);
        if sentences.is_empty() {
            return Vec::new();
        }

        let custom_stopwords: HashSet<String> = config
            .custom_stopwords
            .iter()
            .map(|word| word.to_lowercase())
            .collect();

        let mut sentence_tokens: Vec<Vec<TokenInfo>> = Vec::new();
        for sentence in sentences {
            let sentence_terms = tokenize_sentence(&sentence, config, &custom_stopwords);
            if !sentence_terms.is_empty() {
                sentence_tokens.push(sentence_terms);
            }
        }

        if sentence_tokens.is_empty() {
            return Vec::new();
        }

        let mut term_stats: HashMap<String, TermStats> = HashMap::new();
        let mut position = 0usize;

        for (sentence_idx, tokens) in sentence_tokens.iter().enumerate() {
            for (token_idx, token) in tokens.iter().enumerate() {
                let entry = term_stats.entry(token.term.clone()).or_default();
                entry.count += 1;
                if token.is_cased {
                    entry.cased_count += 1;
                }
                entry.positions.push(position);
                entry.sentences.insert(sentence_idx);
                if token_idx > 0 {
                    entry.neighbors.insert(tokens[token_idx - 1].term.clone());
                }
                if token_idx + 1 < tokens.len() {
                    entry.neighbors.insert(tokens[token_idx + 1].term.clone());
                }
                position += 1;
            }
        }

        let total_tokens = position.max(1);
        let total_sentences = sentence_tokens.len().max(1);
        let max_count = term_stats
            .values()
            .map(|stats| stats.count)
            .max()
            .unwrap_or(1);
        let position_denominator = total_tokens.saturating_sub(1).max(1) as f32;

        let mut word_scores: HashMap<String, f32> = HashMap::new();
        for (term, stats) in &term_stats {
            let t_case = stats.cased_count as f32 / stats.count as f32;
            let t_freq = stats.count as f32 / max_count as f32;
            let t_rel = (stats.neighbors.len() as f32).max(1.0);
            let t_sentence = stats.sentences.len() as f32 / total_sentences as f32;
            let t_position = median_position(&stats.positions) / position_denominator;
            let denominator = t_case + (t_freq / t_rel) + (t_sentence / t_rel);
            // Lower is better; add epsilon to avoid division by zero.
            let score = (t_rel * t_position) / denominator.max(EPSILON);
            word_scores.insert(term.clone(), score);
        }

        let mut phrase_scores: HashMap<String, f32> = HashMap::new();
        for tokens in &sentence_tokens {
            let len = tokens.len();
            for start in 0..len {
                for n in 1..=3 {
                    let end = start + n;
                    if end > len {
                        break;
                    }

                    let mut score = 1.0f32;
                    let mut valid = true;
                    for token in &tokens[start..end] {
                        if let Some(word_score) = word_scores.get(&token.term) {
                            score *= *word_score;
                        } else {
                            valid = false;
                            break;
                        }
                    }
                    if !valid {
                        continue;
                    }

                    let phrase = tokens[start..end]
                        .iter()
                        .map(|token| token.term.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    phrase_scores
                        .entry(phrase)
                        .and_modify(|existing| {
                            if score < *existing {
                                *existing = score;
                            }
                        })
                        .or_insert(score);
                }
            }
        }

        let candidates = phrase_scores
            .into_iter()
            .map(|(keyword, score)| KeywordScore { keyword, score })
            .collect();

        dedupe_keywords(candidates, self.max_keywords)
    }
}

impl Default for YakeExtractor {
    fn default() -> Self {
        Self {
            max_keywords: DEFAULT_MAX_KEYWORDS,
        }
    }
}

#[derive(Debug, Clone)]
struct TokenInfo {
    term: String,
    is_cased: bool,
}

#[derive(Default)]
struct TermStats {
    count: usize,
    cased_count: usize,
    positions: Vec<usize>,
    neighbors: HashSet<String>,
    sentences: HashSet<usize>,
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?' | '\n') {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_string());
            }
            current.clear();
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        sentences.push(trimmed.to_string());
    }

    sentences
}

fn tokenize_sentence(
    sentence: &str,
    config: &AnalyzeConfig,
    custom_stopwords: &HashSet<String>,
) -> Vec<TokenInfo> {
    sentence
        .unicode_words()
        .filter_map(|word| {
            if !is_valid_yake_term(word, config) {
                return None;
            }
            let lower = word.to_lowercase();
            if is_stopword(&lower) || custom_stopwords.contains(&lower) {
                return None;
            }
            let is_cased = is_uppercase(word) || is_capitalized(word);
            Some(TokenInfo {
                term: lower,
                is_cased,
            })
        })
        .collect()
}

fn is_valid_yake_term(term: &str, config: &AnalyzeConfig) -> bool {
    let len = term.chars().count();
    if len < config.min_term_length || len > config.max_term_length {
        return false;
    }

    if term.chars().all(|c| c.is_numeric()) {
        return false;
    }

    let digit_count = term.chars().filter(|c| c.is_numeric()).count();
    if len >= 3 {
        let digit_ratio = digit_count as f32 / len as f32;
        if digit_ratio > config.max_digit_ratio {
            return false;
        }
    }

    if term.is_ascii() && !term.chars().any(|c| c.is_numeric()) {
        let lower = term.to_ascii_lowercase();
        let has_vowel = lower
            .chars()
            .any(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y'));
        if !has_vowel {
            return false;
        }
    }

    true
}

fn is_uppercase(word: &str) -> bool {
    let mut has_alpha = false;
    for ch in word.chars() {
        if ch.is_alphabetic() {
            has_alpha = true;
            if !ch.is_uppercase() {
                return false;
            }
        }
    }
    has_alpha
}

fn is_capitalized(word: &str) -> bool {
    let mut chars = word.chars();
    let first = match chars.next() {
        Some(ch) => ch,
        None => return false,
    };

    let mut has_alpha = first.is_alphabetic();
    if !first.is_uppercase() {
        return false;
    }

    for ch in chars {
        if ch.is_alphabetic() {
            has_alpha = true;
            if ch.is_uppercase() {
                return false;
            }
        }
    }

    has_alpha
}

fn median_position(positions: &[usize]) -> f32 {
    if positions.is_empty() {
        return 0.0;
    }

    let mut sorted = positions.to_vec();
    sorted.sort_unstable();
    let mid = sorted.len() / 2;

    if sorted.len() % 2 == 0 {
        let left = sorted[mid - 1] as f32;
        let right = sorted[mid] as f32;
        (left + right) / 2.0
    } else {
        sorted[mid] as f32
    }
}

fn dedupe_keywords(mut candidates: Vec<KeywordScore>, max_keywords: usize) -> Vec<KeywordScore> {
    candidates.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(Ordering::Equal));

    let mut deduped: Vec<KeywordScore> = Vec::new();
    for candidate in candidates {
        if deduped.len() >= max_keywords {
            break;
        }

        let is_similar = deduped
            .iter()
            .any(|existing| levenshtein_similarity(&existing.keyword, &candidate.keyword) > 0.8);
        if !is_similar {
            deduped.push(candidate);
        }
    }

    deduped
}

fn levenshtein_similarity(a: &str, b: &str) -> f32 {
    let max_len = a.chars().count().max(b.chars().count());
    if max_len == 0 {
        return 1.0;
    }
    let distance = levenshtein_distance(a, b);
    1.0 - (distance as f32 / max_len as f32)
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    if a_chars.is_empty() {
        return b_chars.len();
    }
    if b_chars.is_empty() {
        return a_chars.len();
    }

    let mut prev: Vec<usize> = (0..=b_chars.len()).collect();
    let mut curr: Vec<usize> = vec![0; b_chars.len() + 1];

    for (i, a_ch) in a_chars.iter().enumerate() {
        curr[0] = i + 1;
        for (j, b_ch) in b_chars.iter().enumerate() {
            let substitution = prev[j] + if a_ch == b_ch { 0 } else { 1 };
            let insertion = curr[j] + 1;
            let deletion = prev[j + 1] + 1;
            curr[j + 1] = substitution.min(insertion).min(deletion);
        }
        prev.clone_from_slice(&curr);
    }

    prev[b_chars.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AnalyzeConfig;

    #[test]
    fn test_split_sentences_basic() {
        let text = "First sentence. Second sentence!\nThird sentence?";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 3);
    }

    #[test]
    fn test_extract_keywords_contains_rust() {
        let text =
            "Rust language empowers developers. Rust enforces safety. Memory safety matters.";
        let keywords = YakeExtractor::default().extract(text, &AnalyzeConfig::default());
        assert!(!keywords.is_empty());
        assert!(keywords.iter().any(|kw| kw.keyword == "rust"));
        for window in keywords.windows(2) {
            assert!(window[0].score <= window[1].score);
        }
    }

    #[test]
    fn test_dedup_similar_phrases() {
        let text = "Neural network models. Neural networks scale.";
        let keywords = YakeExtractor::default().extract(text, &AnalyzeConfig::default());
        let has_singular = keywords.iter().any(|kw| kw.keyword == "neural network");
        let has_plural = keywords.iter().any(|kw| kw.keyword == "neural networks");
        assert!(has_singular || has_plural);
        assert!(!(has_singular && has_plural));
    }
}
