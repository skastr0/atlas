use crate::cache::tantivy_backend;
use crate::types::{
    FileFeatures, FileType, SearchFilters, SearchHighlight, SearchHighlightRange,
    SearchQueryMetadata, SearchResultItem, SearchResultsEnvelope, SEARCH_RESULTS_CONTRACT_VERSION,
};
use crate::LogLevel;
use anyhow::{Context, Result};
use std::cmp::Ordering;
use std::ops::Range;
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, Query, QueryParser, TermQuery};
use tantivy::schema::{Field, IndexRecordOption, Schema, Value};
use tantivy::snippet::{Snippet, SnippetGenerator};
use tantivy::{DocAddress, Searcher, TantivyDocument, Term};

const CMAP_DIR: &str = ".cmap";

// These boosts are the whole lexical ranking policy: titles win first, then path text,
// then stored snippets, and finally full-body matches.
const TITLE_BOOST: f32 = 3.0;
const PATH_TEXT_BOOST: f32 = 2.0;
const SNIPPET_BOOST: f32 = 1.5;
const BODY_BOOST: f32 = 1.0;
const EXCERPT_MAX_CHARS: usize = 120;

pub fn run(
    root: &Path,
    query: &str,
    path_filters: &[String],
    type_filters: &[String],
    ext_filters: &[String],
    json: bool,
    explain: bool,
    limit: usize,
    log_level: LogLevel,
) -> Result<()> {
    let cmap_path = root.join(CMAP_DIR);
    let index_dir = tantivy_backend::index_dir(&cmap_path);
    let filters = normalized_filters(path_filters, type_filters, ext_filters);

    let index = tantivy_backend::open_index(&index_dir)?;
    let reader = index.reader().context("Failed to create index reader")?;
    let searcher = reader.searcher();
    let fields = SearchFields::from_schema(&index.schema());

    let mut query_parser = QueryParser::for_index(
        &index,
        vec![fields.title, fields.path_text, fields.snippet, fields.body],
    );
    configure_query_parser(&mut query_parser, fields);

    let parsed_query = query_parser
        .parse_query(query)
        .context("Failed to parse query")?;
    let snippet_generators = SearchSnippetGenerators::create(&searcher, &*parsed_query, fields)?;
    let search_query = compose_query(
        parsed_query,
        fields.scope_terms,
        fields.file_type,
        fields.extension,
        &filters,
    );

    let mut results = Vec::new();
    let total_docs = searcher.num_docs() as usize;
    if limit > 0 && total_docs > 0 {
        let top_docs = searcher.search(&*search_query, &TopDocs::with_limit(total_docs))?;

        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            if let Some(result) = search_result_from_document(
                score,
                doc_address,
                &retrieved_doc,
                &searcher,
                &*search_query,
                &snippet_generators,
                fields.features,
                explain,
            )? {
                results.push(result);
            }
        }
    }

    sort_results_by_rank(&mut results);
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
        render_human_results(query, &envelope.results, log_level);
    }

    Ok(())
}

#[derive(Clone, Copy)]
struct SearchFields {
    path_text: Field,
    scope_terms: Field,
    title: Field,
    snippet: Field,
    body: Field,
    file_type: Field,
    extension: Field,
    features: Field,
}

impl SearchFields {
    fn from_schema(schema: &Schema) -> Self {
        Self {
            path_text: schema.get_field("path_text").unwrap(),
            scope_terms: schema.get_field("scope_terms").unwrap(),
            title: schema.get_field("title").unwrap(),
            snippet: schema.get_field("snippet").unwrap(),
            body: schema.get_field("body").unwrap(),
            file_type: schema.get_field("file_type").unwrap(),
            extension: schema.get_field("extension").unwrap(),
            features: schema.get_field("features").unwrap(),
        }
    }
}

struct SearchSnippetGenerators {
    title: SnippetGenerator,
    body: SnippetGenerator,
    path: SnippetGenerator,
}

impl SearchSnippetGenerators {
    fn create(searcher: &Searcher, query: &dyn Query, fields: SearchFields) -> Result<Self> {
        let mut title = SnippetGenerator::create(searcher, query, fields.title)
            .context("Failed to create title snippet generator")?;
        title.set_max_num_chars(EXCERPT_MAX_CHARS);

        let mut body = SnippetGenerator::create(searcher, query, fields.body)
            .context("Failed to create body snippet generator")?;
        body.set_max_num_chars(EXCERPT_MAX_CHARS);

        let mut path = SnippetGenerator::create(searcher, query, fields.path_text)
            .context("Failed to create path snippet generator")?;
        path.set_max_num_chars(EXCERPT_MAX_CHARS);

        Ok(Self { title, body, path })
    }
}

fn configure_query_parser(query_parser: &mut QueryParser, fields: SearchFields) {
    query_parser.set_conjunction_by_default();
    query_parser.set_field_boost(fields.title, TITLE_BOOST);
    query_parser.set_field_boost(fields.path_text, PATH_TEXT_BOOST);
    query_parser.set_field_boost(fields.snippet, SNIPPET_BOOST);
    query_parser.set_field_boost(fields.body, BODY_BOOST);
}

fn search_result_from_document(
    score: f32,
    doc_address: DocAddress,
    retrieved_doc: &TantivyDocument,
    searcher: &Searcher,
    search_query: &dyn Query,
    snippet_generators: &SearchSnippetGenerators,
    features_field: Field,
    explain: bool,
) -> Result<Option<SearchResultItem>> {
    let Some(features_val) = retrieved_doc.get_first(features_field) else {
        return Ok(None);
    };
    let Some(bytes) = features_val.as_bytes() else {
        return Ok(None);
    };
    let Ok(features) = serde_json::from_slice::<FileFeatures>(bytes) else {
        return Ok(None);
    };

    let title_snippet = snippet_generators.title.snippet_from_doc(retrieved_doc);
    let body_snippet = snippet_generators.body.snippet_from_doc(retrieved_doc);
    let path_snippet = snippet_generators.path.snippet_from_doc(retrieved_doc);
    let matched_fields = matched_fields(&title_snippet, &body_snippet, &path_snippet);
    let highlight = select_highlight(&features, &body_snippet, &title_snippet);
    let reasons = reasons_from_fields(&matched_fields);
    let explanation = if explain {
        Some(serde_json::to_value(
            search_query.explain(searcher, doc_address)?,
        )?)
    } else {
        None
    };

    Ok(Some(SearchResultItem {
        score,
        path: features.path.to_string_lossy().into_owned(),
        file_type: features.file_type,
        extension: tantivy_backend::normalized_extension(&features.path),
        title: features.title,
        snippet: features.snippet,
        matched_fields,
        highlight,
        reasons,
        explanation,
    }))
}

fn matched_fields(
    title_snippet: &Snippet,
    body_snippet: &Snippet,
    path_snippet: &Snippet,
) -> Vec<String> {
    let mut fields = Vec::new();
    if !title_snippet.is_empty() {
        fields.push("title".to_string());
    }
    if !body_snippet.is_empty() {
        fields.push("body".to_string());
    }
    if !path_snippet.is_empty() {
        fields.push("path".to_string());
    }
    fields
}

fn reasons_from_fields(matched_fields: &[String]) -> Vec<String> {
    matched_fields
        .iter()
        .map(|field| format!("{field} match"))
        .collect()
}

fn select_highlight(
    features: &FileFeatures,
    body_snippet: &Snippet,
    title_snippet: &Snippet,
) -> SearchHighlight {
    if !body_snippet.is_empty() {
        return highlight_from_snippet("body", body_snippet);
    }
    if !title_snippet.is_empty() {
        return highlight_from_snippet("title", title_snippet);
    }
    if !features.snippet.trim().is_empty() {
        return fallback_highlight("snippet", &features.snippet);
    }
    fallback_highlight("title", &features.title)
}

fn highlight_from_snippet(field: &str, snippet: &Snippet) -> SearchHighlight {
    let (text, ranges) = normalize_highlight_fragment(snippet.fragment(), snippet.highlighted());

    SearchHighlight {
        field: field.to_string(),
        html: render_html_highlight(&text, &ranges),
        text,
        ranges,
        fallback: false,
    }
}

fn fallback_highlight(field: &str, text: &str) -> SearchHighlight {
    let text = compact_excerpt(text);
    SearchHighlight {
        field: field.to_string(),
        html: escape_html(&text),
        text,
        ranges: Vec::new(),
        fallback: true,
    }
}

fn normalize_highlight_fragment(
    fragment: &str,
    highlighted: &[Range<usize>],
) -> (String, Vec<SearchHighlightRange>) {
    let trim_start = fragment.len() - fragment.trim_start().len();
    let trimmed = fragment.trim();
    let trim_end = trim_start + trimmed.len();

    let ranges: Vec<SearchHighlightRange> = highlighted
        .iter()
        .filter_map(|range| {
            let start = range.start.max(trim_start);
            let end = range.end.min(trim_end);
            if start >= end {
                None
            } else {
                Some(SearchHighlightRange {
                    start: start - trim_start,
                    end: end - trim_start,
                })
            }
        })
        .collect();

    collapse_whitespace_ranges(trimmed, &ranges)
}

fn collapse_whitespace_ranges(
    text: &str,
    ranges: &[SearchHighlightRange],
) -> (String, Vec<SearchHighlightRange>) {
    let mut normalized = String::new();
    let mut byte_map = vec![0usize; text.len() + 1];
    let mut pending_space = false;

    for (idx, ch) in text.char_indices() {
        let space_prefix = if pending_space && !ch.is_whitespace() && !normalized.is_empty() {
            1
        } else {
            0
        };
        byte_map[idx] = normalized.len() + space_prefix;

        if ch.is_whitespace() {
            if !normalized.is_empty() {
                pending_space = true;
            }
            byte_map[idx + ch.len_utf8()] = normalized.len();
            continue;
        }

        if pending_space && !normalized.is_empty() {
            normalized.push(' ');
            pending_space = false;
        }

        normalized.push(ch);
        byte_map[idx + ch.len_utf8()] = normalized.len();
    }
    byte_map[text.len()] = normalized.len();

    let normalized_ranges = ranges
        .iter()
        .filter_map(|range| {
            let start = *byte_map.get(range.start)?;
            let end = *byte_map.get(range.end)?;
            if start >= end {
                None
            } else {
                Some(SearchHighlightRange { start, end })
            }
        })
        .collect();

    (normalized, normalized_ranges)
}

fn render_human_results(query: &str, results: &[SearchResultItem], log_level: LogLevel) {
    if log_level != LogLevel::Quiet {
        if results.is_empty() {
            println!("No results found for '{}'", query);
            return;
        }
        println!("Search results for '{}':\n", query);
    }

    for (index, result) in results.iter().enumerate() {
        println!("{}. {}", index + 1, result.path);
        println!("   Score: {:.4}", result.score);
        println!("   Type: {}", result.file_type.search_term());
        println!("   Title: {}", result.title);
        println!(
            "   Excerpt: {}",
            render_terminal_highlight(&result.highlight)
        );
        if let Some(explanation) = &result.explanation {
            println!("   Explanation:");
            let pretty = serde_json::to_string_pretty(explanation)
                .unwrap_or_else(|_| explanation.to_string());
            for line in pretty.lines() {
                println!("     {}", line);
            }
        }
        println!();
    }
}

fn render_terminal_highlight(highlight: &SearchHighlight) -> String {
    if highlight.ranges.is_empty() {
        return highlight.text.clone();
    }

    let mut rendered = String::new();
    let mut start = 0;
    for range in &highlight.ranges {
        if range.start > highlight.text.len()
            || range.end > highlight.text.len()
            || range.start >= range.end
        {
            continue;
        }
        if start > range.start {
            continue;
        }

        rendered.push_str(&highlight.text[start..range.start]);
        rendered.push_str("**");
        rendered.push_str(&highlight.text[range.start..range.end]);
        rendered.push_str("**");
        start = range.end;
    }
    rendered.push_str(&highlight.text[start..]);
    rendered
}

fn render_html_highlight(text: &str, ranges: &[SearchHighlightRange]) -> String {
    if ranges.is_empty() {
        return escape_html(text);
    }

    let mut rendered = String::new();
    let mut start = 0;
    for range in ranges {
        if range.start > text.len() || range.end > text.len() || range.start >= range.end {
            continue;
        }
        if start > range.start {
            continue;
        }

        rendered.push_str(&escape_html(&text[start..range.start]));
        rendered.push_str("<b>");
        rendered.push_str(&escape_html(&text[range.start..range.end]));
        rendered.push_str("</b>");
        start = range.end;
    }
    rendered.push_str(&escape_html(&text[start..]));
    rendered
}

fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn compact_excerpt(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = collapsed.chars();
    let excerpt: String = chars.by_ref().take(EXCERPT_MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{}...", excerpt.trim_end())
    } else {
        excerpt
    }
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
    scope_terms_field: Field,
    file_type_field: Field,
    extension_field: Field,
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

fn any_term_filter(field: Field, values: &[String]) -> Option<Box<dyn Query>> {
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

/// Search ranking is part of the product contract: lexical score desc, then path asc.
fn sort_results_by_rank(results: &mut [SearchResultItem]) {
    results.sort_by(compare_results_by_rank);
}

fn compare_results_by_rank(left: &SearchResultItem, right: &SearchResultItem) -> Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.path.cmp(&right.path))
}

#[cfg(test)]
mod tests {
    use super::{compare_results_by_rank, fallback_highlight, render_terminal_highlight};
    use crate::types::{FileType, SearchHighlightRange, SearchResultItem};
    use tantivy::collector::TopDocs;
    use tantivy::doc;
    use tantivy::query::QueryParser;
    use tantivy::schema::{Schema, STORED, TEXT};
    use tantivy::snippet::SnippetGenerator;
    use tantivy::{Index, TantivyDocument};

    #[test]
    fn rank_policy_prefers_score_then_path() {
        let a = SearchResultItem {
            score: 1.0,
            path: "beta/file.md".to_string(),
            file_type: FileType::Markdown,
            extension: "md".to_string(),
            title: "Beta".to_string(),
            snippet: String::new(),
            matched_fields: vec!["body".to_string()],
            highlight: fallback_highlight("snippet", "beta"),
            reasons: vec!["body match".to_string()],
            explanation: None,
        };
        let b = SearchResultItem {
            score: 1.0,
            path: "alpha/file.md".to_string(),
            file_type: FileType::Markdown,
            extension: "md".to_string(),
            title: "Alpha".to_string(),
            snippet: String::new(),
            matched_fields: vec!["body".to_string()],
            highlight: fallback_highlight("snippet", "alpha"),
            reasons: vec!["body match".to_string()],
            explanation: None,
        };

        assert_eq!(compare_results_by_rank(&a, &b), std::cmp::Ordering::Greater);
        assert_eq!(compare_results_by_rank(&b, &a), std::cmp::Ordering::Less);
    }

    #[test]
    fn terminal_highlight_marks_ranges() {
        let highlight = crate::types::SearchHighlight {
            field: "body".to_string(),
            text: "rust programming".to_string(),
            html: "rust <b>programming</b>".to_string(),
            ranges: vec![SearchHighlightRange { start: 5, end: 16 }],
            fallback: false,
        };

        assert_eq!(
            render_terminal_highlight(&highlight),
            "rust **programming**"
        );
    }

    #[test]
    fn matched_fields_support_title_only_hits() {
        let mut schema_builder = Schema::builder();
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let path_text = schema_builder.add_text_field("path_text", TEXT | STORED);
        let body = schema_builder.add_text_field("body", TEXT | STORED);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema);
        let mut writer = index.writer(50_000_000).expect("create writer");
        writer
            .add_document(doc!(
                title => "Orbit Search Title Hit",
                path_text => "alpha title only md",
                body => "Nothing interesting in the body"
            ))
            .expect("add document");
        writer.commit().expect("commit document");

        let reader = index.reader().expect("create reader");
        let searcher = reader.searcher();
        let query = QueryParser::for_index(&index, vec![title, path_text, body])
            .parse_query("orbit")
            .expect("parse query");
        let doc_address = searcher
            .search(&query, &TopDocs::with_limit(1))
            .expect("search docs")[0]
            .1;
        let doc = searcher
            .doc::<TantivyDocument>(doc_address)
            .expect("load doc");

        let title_snippet = SnippetGenerator::create(&searcher, &*query, title)
            .expect("title snippet generator")
            .snippet_from_doc(&doc);
        let body_snippet = SnippetGenerator::create(&searcher, &*query, body)
            .expect("body snippet generator")
            .snippet_from_doc(&doc);
        let path_snippet = SnippetGenerator::create(&searcher, &*query, path_text)
            .expect("path snippet generator")
            .snippet_from_doc(&doc);

        assert_eq!(
            super::matched_fields(&title_snippet, &body_snippet, &path_snippet),
            vec!["title".to_string()]
        );
    }
}
