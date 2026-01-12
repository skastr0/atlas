//! RAKE (Rapid Automatic Keyword Extraction)

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use unicode_segmentation::UnicodeSegmentation;

use crate::config::AnalyzeConfig;
use crate::types::PhraseScore;

use super::{clean_text, is_stopword, is_valid_term};

/// Extracts ranked keyword phrases using stopword boundaries.
pub struct RakeExtractor {
    top_n: usize,
}

impl RakeExtractor {
    pub fn new(top_n: usize) -> Self {
        Self { top_n }
    }

    pub fn extract(&self, text: &str, config: &AnalyzeConfig) -> Vec<PhraseScore> {
        if self.top_n == 0 {
            return Vec::new();
        }

        let cleaned = clean_text(text);
        let raw_tokens: Vec<String> = cleaned.unicode_words().map(|w| w.to_lowercase()).collect();

        if raw_tokens.is_empty() {
            return Vec::new();
        }

        let custom_stopwords: HashSet<String> = config
            .custom_stopwords
            .iter()
            .map(|word| word.to_lowercase())
            .collect();

        let phrases = split_phrases(&raw_tokens, config, &custom_stopwords);
        if phrases.is_empty() {
            return Vec::new();
        }

        let mut frequencies: HashMap<String, usize> = HashMap::new();
        let mut degrees: HashMap<String, usize> = HashMap::new();

        for phrase in &phrases {
            let phrase_len = phrase.len();
            if phrase_len == 0 {
                continue;
            }

            for word in phrase {
                *frequencies.entry(word.clone()).or_insert(0) += 1;
                *degrees.entry(word.clone()).or_insert(0) += phrase_len;
            }
        }

        let mut word_scores: HashMap<String, f32> = HashMap::new();
        for (word, freq) in frequencies {
            let degree = degrees.get(&word).copied().unwrap_or(0);
            let score = if freq == 0 {
                0.0
            } else {
                degree as f32 / freq as f32
            };
            word_scores.insert(word, score);
        }

        let mut phrase_scores: HashMap<String, f32> = HashMap::new();
        for phrase in phrases {
            if phrase.is_empty() {
                continue;
            }

            let phrase_text = phrase.join(" ");
            if phrase_scores.contains_key(&phrase_text) {
                continue;
            }

            let mut score = 0.0;
            for word in &phrase {
                if let Some(word_score) = word_scores.get(word) {
                    score += word_score;
                }
            }
            phrase_scores.insert(phrase_text, score);
        }

        let mut scored_phrases: Vec<PhraseScore> = phrase_scores
            .into_iter()
            .map(|(phrase, score)| PhraseScore { phrase, score })
            .collect();

        scored_phrases.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.phrase.cmp(&b.phrase))
        });
        scored_phrases.truncate(self.top_n);
        scored_phrases
    }
}

fn split_phrases(
    tokens: &[String],
    config: &AnalyzeConfig,
    custom_stopwords: &HashSet<String>,
) -> Vec<Vec<String>> {
    let mut phrases = Vec::new();
    let mut current: Vec<String> = Vec::new();

    for token in tokens {
        let is_stop = is_stopword(token) || custom_stopwords.contains(token);
        // A token is a delimiter if it's a stopword OR it's not a valid term (e.g. too short, number)
        if is_stop || !is_valid_term(token, config) {
            if !current.is_empty() {
                phrases.push(current);
                current = Vec::new();
            }
            continue;
        }

        current.push(token.clone());
    }

    if !current.is_empty() {
        phrases.push(current);
    }

    phrases
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_basic_phrase_score() {
        let config = AnalyzeConfig::default();
        let extractor = RakeExtractor::new(5);

        let phrases = extractor.extract("alpha beta gamma", &config);

        assert_eq!(phrases.len(), 1);
        assert_eq!(phrases[0].phrase, "alpha beta gamma");
        assert!((phrases[0].score - 9.0).abs() < 1e-6);
    }

    #[test]
    fn test_extract_splits_on_stopwords() {
        let config = AnalyzeConfig::default();
        let extractor = RakeExtractor::new(5);

        let phrases = extractor.extract("alpha and beta gamma", &config);

        assert_eq!(phrases[0].phrase, "beta gamma");
        assert!(phrases.iter().any(|p| p.phrase == "alpha"));
    }

    #[test]
    fn test_custom_stopwords_split_phrases() {
        let mut config = AnalyzeConfig::default();
        config.custom_stopwords = vec!["beta".to_string()];
        let extractor = RakeExtractor::new(5);

        let phrases = extractor.extract("alpha beta gamma", &config);

        assert!(phrases.iter().any(|p| p.phrase == "alpha"));
        assert!(phrases.iter().any(|p| p.phrase == "gamma"));
    }
}
