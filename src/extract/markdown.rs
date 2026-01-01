//! Markdown text extraction

use super::{parse, ExtractedContent, Language};
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;
use tree_sitter::{Node, Parser};

/// Extract content from Markdown file
pub fn extract_markdown(path: &Path) -> Result<ExtractedContent> {
    let text = fs::read_to_string(path)?;

    let extracted = extract_with_treesitter(&text).unwrap_or_else(|| ExtractedMarkdown {
        title: extract_title_regex(&text),
        headings: extract_headings_regex(&text),
        links: extract_links_regex(&text),
    });

    Ok(ExtractedContent {
        text,
        title: extracted.title,
        headings: extracted.headings,
        links: extracted.links,
        success: true,
    })
}

#[derive(Debug, Default)]
struct ExtractedMarkdown {
    title: Option<String>,
    headings: Vec<String>,
    links: Vec<String>,
}

fn extract_with_treesitter(text: &str) -> Option<ExtractedMarkdown> {
    let tree = parse(text, Language::Markdown)?;
    let root = tree.root_node();
    let mut inline_parser = Parser::new();
    if inline_parser
        .set_language(&tree_sitter_md::INLINE_LANGUAGE.into())
        .is_err()
    {
        return None;
    }

    let mut extracted = ExtractedMarkdown::default();
    let mut saw_wiki_links = false;

    walk_block_tree(
        root,
        text,
        &mut extracted,
        &mut inline_parser,
        &mut saw_wiki_links,
    );

    if !saw_wiki_links {
        // tree-sitter markdown does not always expose wiki links
        extracted.links.extend(scan_wiki_links(text));
    }

    Some(extracted)
}

fn walk_block_tree(
    node: Node,
    source: &str,
    extracted: &mut ExtractedMarkdown,
    inline_parser: &mut Parser,
    saw_wiki_links: &mut bool,
) {
    match node.kind() {
        "atx_heading" => {
            if let Some(level) = atx_heading_level(node) {
                if let Some(text) = extract_heading_text(node, source) {
                    if level == 1 && extracted.title.is_none() {
                        extracted.title = Some(text.clone());
                    }
                    extracted.headings.push(text);
                }
            }
        }
        "inline" | "pipe_table_cell" => {
            let inline_text = slice_node(source, node);
            extract_links_from_inline(inline_text, inline_parser, &mut extracted.links, saw_wiki_links);
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_block_tree(child, source, extracted, inline_parser, saw_wiki_links);
    }
}

fn extract_links_from_inline(
    text: &str,
    inline_parser: &mut Parser,
    links: &mut Vec<String>,
    saw_wiki_links: &mut bool,
) {
    let tree = match inline_parser.parse(text, None) {
        Some(tree) => tree,
        None => return,
    };

    walk_inline_tree(tree.root_node(), text, links, saw_wiki_links);
}

fn walk_inline_tree(node: Node, source: &str, links: &mut Vec<String>, saw_wiki_links: &mut bool) {
    match node.kind() {
        "link_destination" => {
            if is_inline_link_destination(node) {
                if let Some(link) = clean_link_target(slice_node(source, node)) {
                    links.push(link);
                }
            }
        }
        "uri_autolink" | "email_autolink" => {
            if let Some(link) = clean_link_target(slice_node(source, node)) {
                links.push(link);
            }
        }
        "wiki_link" => {
            if let Some(link) = clean_wiki_link(slice_node(source, node)) {
                *saw_wiki_links = true;
                links.push(link);
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_inline_tree(child, source, links, saw_wiki_links);
    }
}

fn atx_heading_level(node: Node) -> Option<u8> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(level) = atx_marker_level(child.kind()) {
            return Some(level);
        }
    }
    None
}

fn atx_marker_level(kind: &str) -> Option<u8> {
    match kind {
        "atx_h1_marker" => Some(1),
        "atx_h2_marker" => Some(2),
        "atx_h3_marker" => Some(3),
        "atx_h4_marker" => Some(4),
        "atx_h5_marker" => Some(5),
        "atx_h6_marker" => Some(6),
        _ => None,
    }
}

fn extract_heading_text(node: Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "inline" | "heading_content" | "text" => {
                let text = slice_node(source, child).trim();
                if !text.is_empty() {
                    return Some(text.to_string());
                }
            }
            _ => {}
        }
    }

    let stripped = strip_atx_marker(slice_node(source, node));
    if stripped.is_empty() {
        None
    } else {
        Some(stripped)
    }
}

fn strip_atx_marker(raw: &str) -> String {
    let trimmed = raw.trim();
    let bytes = trimmed.as_bytes();
    let mut start = 0;

    while start < bytes.len() && bytes[start] == b'#' {
        start += 1;
    }
    while start < bytes.len() && bytes[start].is_ascii_whitespace() {
        start += 1;
    }

    let mut end = bytes.len();
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    while end > start && bytes[end - 1] == b'#' {
        end -= 1;
    }
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }

    if start >= end {
        return String::new();
    }

    trimmed[start..end].to_string()
}

fn slice_node<'a>(source: &'a str, node: Node) -> &'a str {
    let start = node.start_byte();
    let end = node.end_byte();
    source.get(start..end).unwrap_or("")
}

fn clean_link_target(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let cleaned = if trimmed.starts_with('<') && trimmed.ends_with('>') && trimmed.len() >= 2 {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn clean_wiki_link(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let cleaned = trimmed
        .strip_prefix("[[")
        .and_then(|inner| inner.strip_suffix("]]"))
        .unwrap_or(trimmed)
        .trim();

    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn scan_wiki_links(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut links = Vec::new();
    let mut i = 0;

    while i + 1 < bytes.len() {
        if bytes[i] == b'[' && bytes[i + 1] == b'[' {
            let start = i + 2;
            let mut j = start;
            let mut found = false;

            while j + 1 < bytes.len() {
                if bytes[j] == b']' && bytes[j + 1] == b']' {
                    if let Some(slice) = source.get(start..j) {
                        let trimmed = slice.trim();
                        if !trimmed.is_empty() {
                            links.push(trimmed.to_string());
                        }
                    }
                    i = j + 2;
                    found = true;
                    break;
                }
                j += 1;
            }

            if !found {
                break;
            }
            continue;
        }

        i += 1;
    }

    links
}

fn is_inline_link_destination(node: Node) -> bool {
    let parent = match node.parent() {
        Some(parent) => parent,
        None => return false,
    };
    let kind = parent.kind();
    kind == "inline_link" || kind == "link" || kind.contains("inline_link") || kind == "image"
}

fn extract_title_regex(text: &str) -> Option<String> {
    // Match first # heading (not ##)
    let re = Regex::new(r"^#\s+(.+)$").ok()?;
    for line in text.lines() {
        if let Some(caps) = re.captures(line) {
            return Some(caps.get(1)?.as_str().to_string());
        }
    }
    None
}

fn extract_headings_regex(text: &str) -> Vec<String> {
    let mut headings = Vec::new();
    let re = Regex::new(r"^#{1,6}\s+(.+)$").unwrap();

    for line in text.lines() {
        if let Some(caps) = re.captures(line) {
            if let Some(heading) = caps.get(1) {
                headings.push(heading.as_str().to_string());
            }
        }
    }

    headings
}

fn extract_links_regex(text: &str) -> Vec<String> {
    let mut links = Vec::new();

    // Match markdown links [text](url)
    let md_link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    for caps in md_link_re.captures_iter(text) {
        if let Some(url) = caps.get(2) {
            links.push(url.as_str().to_string());
        }
    }

    // Match wiki-style links [[link]]
    let wiki_link_re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    for caps in wiki_link_re.captures_iter(text) {
        if let Some(link) = caps.get(1) {
            links.push(link.as_str().to_string());
        }
    }

    // Match autolinks <https://example.com>
    let autolink_re = Regex::new(r"<([^>]+)>").unwrap();
    for caps in autolink_re.captures_iter(text) {
        if let Some(link) = caps.get(1) {
            links.push(link.as_str().to_string());
        }
    }

    links
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extracts_headings_and_links_with_treesitter() {
        let markdown = "# Title\n\n## Section One\n\nLink: [Example](https://example.com)\n\nWiki: [[Wiki Link]]\n\nAutolink: <https://example.com/auto>\n\n### Subsection\n\n# Second Title\n";
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(markdown.as_bytes()).unwrap();

        let extracted = extract_markdown(file.path()).unwrap();

        assert_eq!(extracted.title, Some("Title".to_string()));
        assert_eq!(
            extracted.headings,
            vec![
                "Title".to_string(),
                "Section One".to_string(),
                "Subsection".to_string(),
                "Second Title".to_string(),
            ]
        );
        assert!(extracted.links.contains(&"https://example.com".to_string()));
        assert!(extracted.links.contains(&"Wiki Link".to_string()));
        assert!(extracted
            .links
            .contains(&"https://example.com/auto".to_string()));
    }
}
