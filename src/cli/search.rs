use crate::cache::tantivy_backend;
use crate::types::{
    FileFeatures, FileType, SearchFilters, SearchQueryMetadata, SearchResultItem,
    SearchResultsEnvelope, SEARCH_RESULTS_CONTRACT_VERSION,
};
use crate::LogLevel;
use anyhow::{Context, Result};
use std::cmp::Ordering;
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, Query, QueryParser, TermQuery};
use tantivy::schema::IndexRecordOption;
use tantivy::schema::Value;
use tantivy::Term;

const CMAP_DIR: &str = ".cmap";
const TITLE_BOOST: f32 = 3.0;
const PATH_TEXT_BOOST: f32 = 2.0;
const SNIPPET_BOOST: f32 = 1.5;
const BODY_BOOST: f32 = 1.0;

pub fn run(
    root: &Path,
    query: &str,
    path_filters: &[String],
    type_filters: &[String],
    ext_filters: &[String],
    json: bool,
    limit: usize,
    log_level: LogLevel,
) -> Result<()> {
    let cmap_path = root.join(CMAP_DIR);
    let index_dir = tantivy_backend::index_dir(&cmap_path);
    let filters = normalized_filters(path_filters, type_filters, ext_filters);

    let index = tantivy_backend::open_index(&index_dir)?;
    let reader = index.reader().context("Failed to create index reader")?;
    let searcher = reader.searcher();

    let schema = index.schema();
    let path_text_field = schema.get_field("path_text").unwrap();
    let scope_terms_field = schema.get_field("scope_terms").unwrap();
    let title_field = schema.get_field("title").unwrap();
    let snippet_field = schema.get_field("snippet").unwrap();
    let body_field = schema.get_field("body").unwrap();
    let file_type_field = schema.get_field("file_type").unwrap();
    let extension_field = schema.get_field("extension").unwrap();
    let features_field = schema.get_field("features").unwrap();

    let mut query_parser = QueryParser::for_index(
        &index,
        vec![title_field, path_text_field, snippet_field, body_field],
    );
    query_parser.set_conjunction_by_default();
    query_parser.set_field_boost(title_field, TITLE_BOOST);
    query_parser.set_field_boost(path_text_field, PATH_TEXT_BOOST);
    query_parser.set_field_boost(snippet_field, SNIPPET_BOOST);
    query_parser.set_field_boost(body_field, BODY_BOOST);

    let parsed_query = query_parser
        .parse_query(query)
        .context("Failed to parse query")?;
    let search_query = compose_query(
        parsed_query,
        scope_terms_field,
        file_type_field,
        extension_field,
        &filters,
    );

    let mut results = Vec::new();
    let total_docs = searcher.num_docs() as usize;
    if limit > 0 && total_docs > 0 {
        let top_docs = searcher.search(&*search_query, &TopDocs::with_limit(total_docs))?;

        for (score, doc_address) in top_docs {
            let retrieved_doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;
            if let Some(features_val) = retrieved_doc.get_first(features_field) {
                if let Some(bytes) = features_val.as_bytes() {
                    if let Ok(features) = serde_json::from_slice::<FileFeatures>(bytes) {
                        results.push(SearchResultItem {
                            score,
                            path: features.path.to_string_lossy().into_owned(),
                            file_type: features.file_type,
                            extension: tantivy_backend::normalized_extension(&features.path),
                            title: features.title,
                            snippet: features.snippet,
                        });
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
    results.truncate(limit);

    let envelope = SearchResultsEnvelope {
        version: SEARCH_RESULTS_CONTRACT_VERSION,
        index_version: tantivy_backend::SEARCH_INDEX_VERSION.to_string(),
        query: SearchQueryMetadata {
            text: query.to_string(),
            limit,
            filters,
        },
        result_count: results.len(),
        results,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        if log_level != LogLevel::Quiet {
            if envelope.results.is_empty() {
                println!("No results found for '{}'", query);
                return Ok(());
            }
            println!("Search results for '{}':\n", query);
        }

        for result in envelope.results {
            println!("[{:.4}] {}", result.score, result.path);
            println!("  Title: {}", result.title);
            let snippet = result.snippet.replace('\n', " ");
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

fn normalized_filters(
    path_filters: &[String],
    type_filters: &[String],
    ext_filters: &[String],
) -> SearchFilters {
    SearchFilters {
        paths: normalize_values(path_filters, |value| {
            tantivy_backend::normalize_scope_filter(value)
        }),
        types: normalize_values(type_filters, |value| normalize_type_filter(value)),
        extensions: normalize_values(ext_filters, |value| {
            tantivy_backend::normalize_extension_filter(value)
        }),
    }
}

fn normalize_values(values: &[String], normalize: impl Fn(&str) -> Option<String>) -> Vec<String> {
    let mut normalized: Vec<String> = values.iter().filter_map(|value| normalize(value)).collect();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn normalize_type_filter(value: &str) -> Option<String> {
    let normalized = FileType::normalize_search_term(value);
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn compose_query(
    parsed_query: Box<dyn Query>,
    scope_terms_field: tantivy::schema::Field,
    file_type_field: tantivy::schema::Field,
    extension_field: tantivy::schema::Field,
    filters: &SearchFilters,
) -> Box<dyn Query> {
    let mut clauses = vec![(Occur::Must, parsed_query)];

    if let Some(path_filter_query) = any_term_filter(scope_terms_field, &filters.paths) {
        clauses.push((Occur::Must, path_filter_query));
    }
    if let Some(type_filter_query) = any_term_filter(file_type_field, &filters.types) {
        clauses.push((Occur::Must, type_filter_query));
    }
    if let Some(extension_filter_query) = any_term_filter(extension_field, &filters.extensions) {
        clauses.push((Occur::Must, extension_filter_query));
    }

    if clauses.len() == 1 {
        clauses.pop().unwrap().1
    } else {
        Box::new(BooleanQuery::new(clauses))
    }
}

fn any_term_filter(field: tantivy::schema::Field, values: &[String]) -> Option<Box<dyn Query>> {
    if values.is_empty() {
        return None;
    }

    let mut queries: Vec<(Occur, Box<dyn Query>)> = values
        .iter()
        .map(|value| {
            let query: Box<dyn Query> = Box::new(TermQuery::new(
                Term::from_field_text(field, value),
                IndexRecordOption::Basic,
            ));
            (Occur::Should, query)
        })
        .collect();

    if queries.len() == 1 {
        queries.pop().map(|(_, query)| query)
    } else {
        Some(Box::new(BooleanQuery::new(queries)))
    }
}
