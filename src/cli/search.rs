use crate::types::FileFeatures;
use crate::LogLevel;
use anyhow::{Context, Result};
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::Value;

const CMAP_DIR: &str = ".cmap";

pub fn run(root: &Path, query: &str, json: bool, limit: usize, log_level: LogLevel) -> Result<()> {
    let cmap_path = root.join(CMAP_DIR);
    let index_dir = cmap_path.join("index/tantivy-v1");

    if !index_dir.exists() {
        anyhow::bail!("Index not found. Run `cmap build` first.");
    }

    let index = tantivy::Index::open_in_dir(&index_dir).context("Failed to open tantivy index")?;
    let reader = index.reader().context("Failed to create index reader")?;
    let searcher = reader.searcher();

    let schema = index.schema();
    let title_field = schema.get_field("title").unwrap();
    let snippet_field = schema.get_field("snippet").unwrap();
    let body_field = schema.get_field("body").unwrap();
    let features_field = schema.get_field("features").unwrap();

    let query_parser = QueryParser::for_index(&index, vec![title_field, snippet_field, body_field]);
    let parsed_query = query_parser
        .parse_query(query)
        .context("Failed to parse query")?;

    let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(limit))?;

    let mut results = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved_doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;
        if let Some(features_val) = retrieved_doc.get_first(features_field) {
            if let Some(bytes) = features_val.as_bytes() {
                if let Ok(features) = serde_json::from_slice::<FileFeatures>(bytes) {
                    results.push((score, features));
                }
            }
        }
    }

    results.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.path.cmp(&b.1.path))
    });

    if json {
        #[derive(serde::Serialize)]
        struct SearchResult<'a> {
            score: f32,
            #[serde(flatten)]
            features: &'a FileFeatures,
        }
        let output: Vec<SearchResult> = results
            .iter()
            .map(|(score, features)| SearchResult {
                score: *score,
                features,
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        if log_level != LogLevel::Quiet {
            if results.is_empty() {
                println!("No results found for '{}'", query);
                return Ok(());
            }
            println!("Search results for '{}':\n", query);
        }

        for (score, features) in results {
            println!("[{:.4}] {}", score, features.path.display());
            println!("  Title: {}", features.title);
            let snippet = features.snippet.replace('\n', " ");
            let snippet = if snippet.len() > 100 {
                format!("{}...", &snippet[..97])
            } else {
                snippet
            };
            println!("  Snippet: {}", snippet);
            println!();
        }
    }

    Ok(())
}
