//! Markdown view rendering
//!
//! Generates:
//! - ROOT_ATLAS.md - Top-level map
//! - Per-folder INDEX.md files
//! - TERMS.md - Term to files mapping

mod atlas;
mod folder_index;
mod graph;
mod term_index;

use std::collections::{HashMap, HashSet};

pub use atlas::*;
pub use folder_index::*;
pub use graph::*;
pub use term_index::*;

#[derive(Debug, Clone, Default)]
pub(crate) struct CodeSymbolSummary {
    pub total: usize,
    pub exported: usize,
    pub kind_counts: Vec<(String, usize)>,
    pub exported_kind_counts: Vec<(String, usize)>,
    pub top_symbols: Vec<String>,
    pub top_exported_symbols: Vec<String>,
}

pub(crate) fn summarize_code_symbols(headings: &[String]) -> CodeSymbolSummary {
    let mut summary = CodeSymbolSummary::default();
    let mut kind_counts: HashMap<String, usize> = HashMap::new();
    let mut exported_counts: HashMap<String, usize> = HashMap::new();
    let mut seen = HashSet::new();
    let mut seen_exported = HashSet::new();

    for heading in headings {
        let Some(parsed) = parse_symbol_heading(heading) else {
            continue;
        };
        if parsed.kind == "impl" {
            continue;
        }

        summary.total += 1;
        *kind_counts.entry(parsed.kind.to_string()).or_insert(0) += 1;

        if parsed.exported {
            summary.exported += 1;
            *exported_counts.entry(parsed.kind.to_string()).or_insert(0) += 1;
        }

        if seen.insert(parsed.name.clone()) {
            summary.top_symbols.push(parsed.name.clone());
        }

        if parsed.exported && seen_exported.insert(parsed.name.clone()) {
            summary.top_exported_symbols.push(parsed.name);
        }
    }

    summary.kind_counts = sort_symbol_counts(kind_counts);
    summary.exported_kind_counts = sort_symbol_counts(exported_counts);
    summary
}

pub(crate) fn format_symbol_counts(counts: &[(String, usize)]) -> String {
    counts
        .iter()
        .map(|(kind, count)| format!("{} {}", count, pluralize_kind(kind, *count)))
        .collect::<Vec<_>>()
        .join(", ")
}

struct ParsedSymbol {
    kind: &'static str,
    name: String,
    exported: bool,
}

fn parse_symbol_heading(heading: &str) -> Option<ParsedSymbol> {
    let base = heading.split(" - ").next().unwrap_or(heading).trim();
    if base.is_empty() {
        return None;
    }

    let mut exported = false;
    let mut rest = base;
    for prefix in ["pub ", "export "] {
        if rest.starts_with(prefix) {
            exported = true;
            rest = rest[prefix.len()..].trim_start();
        }
    }

    let (kind, name) = if let Some(name) = rest.strip_prefix("fn ") {
        ("function", name)
    } else if let Some(name) = rest.strip_prefix("function ") {
        ("function", name)
    } else if let Some(name) = rest.strip_prefix("struct ") {
        ("struct", name)
    } else if let Some(name) = rest.strip_prefix("enum ") {
        ("enum", name)
    } else if let Some(name) = rest.strip_prefix("trait ") {
        ("trait", name)
    } else if let Some(name) = rest.strip_prefix("type ") {
        ("type", name)
    } else if let Some(name) = rest.strip_prefix("class ") {
        ("class", name)
    } else if let Some(name) = rest.strip_prefix("interface ") {
        ("interface", name)
    } else if let Some(name) = rest.strip_prefix("impl ") {
        ("impl", name)
    } else {
        return None;
    };

    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }

    let symbol_name = trimmed
        .split_whitespace()
        .next()
        .unwrap_or(trimmed)
        .to_string();

    Some(ParsedSymbol {
        kind,
        name: symbol_name,
        exported,
    })
}

fn sort_symbol_counts(map: HashMap<String, usize>) -> Vec<(String, usize)> {
    let mut counts: Vec<(String, usize)> = map.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    counts
}

fn pluralize_kind(kind: &str, count: usize) -> String {
    if count == 1 {
        return kind.to_string();
    }

    match kind {
        "class" => "classes".to_string(),
        _ => format!("{}s", kind),
    }
}

#[cfg(test)]
mod tests {
    use super::{format_symbol_counts, summarize_code_symbols};

    #[test]
    fn summarizes_code_symbols_and_exports() {
        let headings = vec![
            "pub fn extract - pub fn extract()".to_string(),
            "struct FileFeatures".to_string(),
            "impl FileFeatures".to_string(),
            "export function build".to_string(),
            "export class Widget".to_string(),
            "interface Shape".to_string(),
        ];

        let summary = summarize_code_symbols(&headings);
        assert_eq!(summary.total, 5);
        assert_eq!(summary.exported, 3);
        assert!(summary
            .kind_counts
            .iter()
            .any(|(kind, count)| kind == "function" && *count == 2));
        assert!(summary
            .kind_counts
            .iter()
            .any(|(kind, count)| kind == "struct" && *count == 1));
        assert!(summary
            .kind_counts
            .iter()
            .any(|(kind, count)| kind == "class" && *count == 1));
        assert!(summary
            .kind_counts
            .iter()
            .any(|(kind, count)| kind == "interface" && *count == 1));
        assert_eq!(
            summary.top_symbols.first().map(|s| s.as_str()),
            Some("extract")
        );

        let export_summary = format_symbol_counts(&summary.exported_kind_counts);
        assert!(!export_summary.is_empty());
    }
}
