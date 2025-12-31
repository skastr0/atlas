//! Unicode-aware word tokenization

use std::sync::OnceLock;

use regex::Regex;
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;

use crate::config::AnalyzeConfig;

fn url_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"https?://\S+|www\.\S+").unwrap())
}

pub fn clean_text(text: &str) -> String {
    url_regex().replace_all(text, " ").into_owned()
}

pub fn is_valid_term(term: &str, config: &AnalyzeConfig) -> bool {
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

/// Tokenize text into lowercase words
pub fn tokenize(text: &str) -> Vec<String> {
    let config = AnalyzeConfig::default();
    tokenize_with_config(text, &config)
}

/// Tokenize and filter with custom minimum length
pub fn tokenize_min_length(text: &str, min_length: usize) -> Vec<String> {
    let mut config = AnalyzeConfig::default();
    config.min_term_length = min_length;
    tokenize_with_config(text, &config)
}

/// Tokenize and filter with analyze configuration
pub fn tokenize_with_config(text: &str, config: &AnalyzeConfig) -> Vec<String> {
    let cleaned = clean_text(text);
    cleaned
        .unicode_words()
        .filter(|word| is_valid_term(word, config))
        .map(|word| word.to_lowercase())
        .collect()
}

/// Check if a word is a stopword (basic English stopwords)
pub fn is_stopword(word: &str) -> bool {
    STOPWORDS.contains(&word.to_lowercase().as_str())
}

/// Filter stopwords from tokens
pub fn filter_stopwords(tokens: Vec<String>, custom_stopwords: &[String]) -> Vec<String> {
    if custom_stopwords.is_empty() {
        return tokens.into_iter().filter(|t| !is_stopword(t)).collect();
    }

    let custom_set: HashSet<String> = custom_stopwords
        .iter()
        .map(|word| word.to_lowercase())
        .collect();

    tokens
        .into_iter()
        .filter(|token| !is_stopword(token) && !custom_set.contains(token))
        .collect()
}

/// Basic English and technical stopwords
const STOPWORDS: &[&str] = &[
    "a", "an", "and", "arent", "are", "argument", "as", "at", "be", "been", "being", "but", "by",
    "came", "can", "cant", "class", "code", "com", "come", "could", "data", "did", "didnt", "do",
    "does", "doing", "done", "dont", "example", "examples", "file", "files", "for", "from",
    "function", "get", "gets", "go", "going", "gone", "got", "had", "has", "have", "having", "he",
    "her", "here", "hers", "herself", "hes", "him", "himself", "his", "how", "html", "http", "https",
    "i", "if", "im", "in", "into", "io", "is", "isnt", "it", "its", "itself", "ive", "just", "know",
    "let", "lets", "look", "make", "made", "makes", "me", "method", "might", "more", "most", "must",
    "my", "myself", "net", "need", "no", "nor", "not", "now", "object", "of", "on", "only", "or",
    "org", "other", "our", "ours", "ourselves", "out", "over", "own", "parameter", "pdf", "put",
    "return", "returns", "said", "say", "see", "seem", "she", "shes", "should", "so", "some",
    "such", "take", "tell", "than", "that", "the", "their", "theirs", "them", "themselves", "then",
    "there", "these", "they", "theyre", "theyve", "think", "this", "those", "through", "to", "told",
    "too", "try", "type", "under", "until", "up", "used", "using", "value", "very", "was", "wasnt",
    "we", "weve", "were", "werent", "what", "when", "where", "which", "while", "who", "whom", "why",
    "will", "with", "wont", "would", "www", "you", "youre", "your", "yours", "yourself",
    "yourselves", "youve", "want",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AnalyzeConfig;

    #[test]
    fn test_clean_text_removes_urls() {
        let input = "Check out https://example.com for more info";
        let output = clean_text(input);
        assert!(!output.contains("https://"));
        assert!(!output.contains("example.com"));
    }

    #[test]
    fn test_clean_text_removes_www() {
        let input = "Visit www.example.com today";
        let output = clean_text(input);
        assert!(!output.contains("www."));
    }

    #[test]
    fn test_clean_text_preserves_normal_text() {
        let input = "This is normal text without URLs";
        let output = clean_text(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_filter_stopwords_with_builtin_list() {
        let tokens = vec![
            "http".to_string(),
            "lets".to_string(),
            "using".to_string(),
            "function".to_string(),
            "signal".to_string(),
        ];
        let filtered = filter_stopwords(tokens, &[]);
        assert_eq!(filtered, vec!["signal"]);
    }

    #[test]
    fn test_filter_stopwords_with_custom_list() {
        let tokens = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let custom = vec!["beta".to_string(), "Gamma".to_string()];
        let filtered = filter_stopwords(tokens, &custom);
        assert_eq!(filtered, vec!["alpha"]);
    }

    #[test]
    fn test_is_valid_term_filters_numbers_and_lengths() {
        let config = AnalyzeConfig::default();
        let long_term = "a".repeat(config.max_term_length + 1);
        assert!(!is_valid_term("2024", &config));
        assert!(!is_valid_term("ai", &config));
        assert!(!is_valid_term(&long_term, &config));
        assert!(is_valid_term("signal", &config));
    }

    #[test]
    fn test_is_valid_term_filters_digit_ratio() {
        let config = AnalyzeConfig::default();
        assert!(!is_valid_term("v1234", &config));
    }

    #[test]
    fn test_is_valid_term_filters_no_vowels() {
        let config = AnalyzeConfig::default();
        assert!(!is_valid_term("tsk", &config));
    }

    #[test]
    fn test_is_valid_term_allows_short_version_token() {
        let mut config = AnalyzeConfig::default();
        config.min_term_length = 2;
        assert!(is_valid_term("v2", &config));
    }
}
