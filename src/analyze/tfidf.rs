//! TF-IDF computation

use std::collections::HashMap;

/// Compute term frequencies for a document
pub fn compute_tf(tokens: &[String]) -> HashMap<String, f32> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for token in tokens {
        *counts.entry(token.clone()).or_insert(0) += 1;
    }

    let total = tokens.len() as f32;
    counts
        .into_iter()
        .map(|(term, count)| (term, count as f32 / total))
        .collect()
}

/// Compute IDF for a term given document frequencies
pub fn compute_idf(total_docs: usize, doc_freq: usize) -> f32 {
    if doc_freq == 0 {
        return 0.0;
    }
    ((total_docs as f32) / (doc_freq as f32)).ln() + 1.0
}

/// Compute TF-IDF scores for a document
pub fn compute_tfidf(
    tf: &HashMap<String, f32>,
    df: &HashMap<String, usize>,
    total_docs: usize,
) -> HashMap<String, f32> {
    tf.iter()
        .map(|(term, &tf_score)| {
            let doc_freq = df.get(term).copied().unwrap_or(1);
            let idf = compute_idf(total_docs, doc_freq);
            (term.clone(), tf_score * idf)
        })
        .collect()
}

/// Get top N terms by TF-IDF score
pub fn top_terms_by_tfidf(tfidf: HashMap<String, f32>, n: usize) -> Vec<(String, f32)> {
    let mut sorted: Vec<_> = tfidf.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sorted.truncate(n);
    sorted
}
