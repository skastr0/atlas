//! N-gram extraction (bigrams, trigrams)

use super::tokenize::is_stopword;
use std::collections::HashMap;

/// Extract bigrams from tokens
pub fn extract_bigrams(tokens: &[String]) -> HashMap<String, usize> {
    let mut bigrams = HashMap::new();

    for window in tokens.windows(2) {
        // Skip if either word is a stopword
        if is_stopword(&window[0]) || is_stopword(&window[1]) {
            continue;
        }

        let bigram = format!("{} {}", window[0], window[1]);
        *bigrams.entry(bigram).or_insert(0) += 1;
    }

    bigrams
}

/// Extract trigrams from tokens
pub fn extract_trigrams(tokens: &[String]) -> HashMap<String, usize> {
    let mut trigrams = HashMap::new();

    for window in tokens.windows(3) {
        // Skip if all words are stopwords (allow some stopwords in middle)
        if is_stopword(&window[0]) && is_stopword(&window[2]) {
            continue;
        }

        let trigram = format!("{} {} {}", window[0], window[1], window[2]);
        *trigrams.entry(trigram).or_insert(0) += 1;
    }

    trigrams
}

/// Get top N n-grams by count
pub fn top_ngrams(ngrams: HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    let mut sorted: Vec<_> = ngrams.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(n);
    sorted
}
