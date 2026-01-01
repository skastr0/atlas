//! Rust source extraction using tree-sitter

use super::treesitter::{parse, Language};
use super::ExtractedContent;
use anyhow::Result;
use std::fs;
use std::path::Path;
use tree_sitter::Node;

/// Extract content from Rust source files.
pub fn extract_rust(path: &Path) -> Result<ExtractedContent> {
    let text = fs::read_to_string(path)?;
    let title = extract_module_doc_title(&text).or_else(|| {
        text.lines()
            .find(|line| !line.trim().is_empty())
            .filter(|line| line.len() < 200)
            .map(|line| line.trim().to_string())
    });

    let mut headings = Vec::new();
    let mut links = Vec::new();
    let mut success = true;

    if let Some(tree) = parse(&text, Language::Rust) {
        let root = tree.root_node();
        collect_nodes(&text, root, &mut headings, &mut links);
    } else {
        success = false;
    }

    Ok(ExtractedContent {
        text,
        title,
        headings,
        links,
        success,
    })
}

fn extract_module_doc_title(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("//!") {
            let title = trimmed.trim_start_matches("//!").trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
            continue;
        }

        // Stop once we hit non-doc content.
        break;
    }

    None
}

fn collect_nodes(source: &str, node: Node, headings: &mut Vec<String>, links: &mut Vec<String>) {
    match node.kind() {
        "function_item" => {
            if let Some(name) = extract_identifier(source, node) {
                let visibility = is_public(node);
                let signature = signature_snippet(source, node);
                headings.push(format_symbol(visibility, "fn", &name, signature.as_deref()));
            }
        }
        "struct_item" => {
            if let Some(name) = extract_identifier(source, node) {
                let visibility = is_public(node);
                headings.push(format_symbol(visibility, "struct", &name, None));
            }
        }
        "enum_item" => {
            if let Some(name) = extract_identifier(source, node) {
                let visibility = is_public(node);
                headings.push(format_symbol(visibility, "enum", &name, None));
            }
        }
        "trait_item" => {
            if let Some(name) = extract_identifier(source, node) {
                let visibility = is_public(node);
                headings.push(format_symbol(visibility, "trait", &name, None));
            }
        }
        "type_item" => {
            if let Some(name) = extract_identifier(source, node) {
                let visibility = is_public(node);
                headings.push(format_symbol(visibility, "type", &name, None));
            }
        }
        "impl_item" => {
            if let Some(type_name) = extract_impl_type_name(source, node) {
                headings.push(format!("impl {}", type_name));
            }
        }
        "use_declaration" => {
            if let Some(target) = extract_use_target(source, node) {
                links.push(target);
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_nodes(source, child, headings, links);
    }
}

fn extract_identifier<'a>(source: &str, node: Node<'a>) -> Option<String> {
    if let Some(name_node) = node.child_by_field_name("name") {
        return node_text(source, name_node).map(|text| text.to_string());
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" || child.kind() == "type_identifier" {
            return node_text(source, child).map(|text| text.to_string());
        }
    }

    None
}

fn extract_impl_type_name<'a>(source: &str, node: Node<'a>) -> Option<String> {
    if let Some(type_node) = node.child_by_field_name("type") {
        return extract_type_label(source, type_node);
    }

    let mut type_nodes = Vec::new();
    collect_descendants(node, &["type_identifier", "scoped_type_identifier"], &mut type_nodes);

    type_nodes
        .last()
        .and_then(|type_node| extract_type_label(source, *type_node))
}

fn extract_type_label<'a>(source: &str, node: Node<'a>) -> Option<String> {
    if node.kind() == "type_identifier" || node.kind() == "scoped_type_identifier" {
        return node_text(source, node).map(|text| text.to_string());
    }

    if let Some(name_node) = find_descendant(node, &["type_identifier", "scoped_type_identifier"]) {
        return node_text(source, name_node).map(|text| text.to_string());
    }

    node_text(source, node).map(|text| normalize_whitespace(text))
}

fn signature_snippet<'a>(source: &str, node: Node<'a>) -> Option<String> {
    let text = node_text(source, node)?;
    let line = text.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }

    let mut normalized = normalize_whitespace(line);
    normalized = normalized.trim_end().trim_end_matches('{').trim_end().to_string();
    Some(normalized)
}

fn extract_use_target<'a>(source: &str, node: Node<'a>) -> Option<String> {
    let text = node_text(source, node)?;
    let normalized = normalize_whitespace(text);
    let trimmed = normalized.trim_end_matches(';').trim();
    let use_pos = trimmed.find("use ")?;
    let target = trimmed[use_pos + 4..].trim();
    if target.is_empty() {
        return None;
    }
    Some(target.to_string())
}

fn format_symbol(visibility: bool, kind: &str, name: &str, signature: Option<&str>) -> String {
    let prefix = if visibility { "pub " } else { "" };
    let base = format!("{}{} {}", prefix, kind, name);
    if let Some(signature) = signature {
        if signature.is_empty() {
            return base;
        }
        return format!("{} - {}", base, signature);
    }
    base
}

fn is_public<'a>(node: Node<'a>) -> bool {
    if node.child_by_field_name("visibility").is_some() {
        return true;
    }

    let mut cursor = node.walk();
    let has_visibility = node
        .children(&mut cursor)
        .any(|child| child.kind() == "visibility_modifier");
    has_visibility
}

fn node_text<'a>(source: &'a str, node: Node<'a>) -> Option<&'a str> {
    source.get(node.start_byte()..node.end_byte())
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn collect_descendants<'a>(node: Node<'a>, kinds: &[&str], output: &mut Vec<Node<'a>>) {
    if kinds.iter().any(|kind| *kind == node.kind()) {
        output.push(node);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_descendants(child, kinds, output);
    }
}

fn find_descendant<'a>(node: Node<'a>, kinds: &[&str]) -> Option<Node<'a>> {
    if kinds.iter().any(|kind| *kind == node.kind()) {
        return Some(node);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_descendant(child, kinds) {
            return Some(found);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::extract_rust;
    use std::path::Path;

    #[test]
    fn extracts_symbols_and_use_links_from_main() {
        let content = extract_rust(Path::new("src/main.rs")).expect("extract main.rs");

        assert!(content.headings.iter().any(|h| h.starts_with("fn main")));
        assert!(content.headings.iter().any(|h| h.starts_with("struct Cli")));
        assert!(content.headings.iter().any(|h| h.starts_with("enum Commands")));
        assert!(content.headings.iter().any(|h| h.starts_with("pub enum LogLevel")));
        assert!(content.headings.iter().any(|h| h.starts_with("impl LogLevel")));
        assert!(content.links.iter().any(|link| link.contains("anyhow::Result")));
    }

    #[test]
    fn extracts_public_symbols_from_types() {
        let content = extract_rust(Path::new("src/types.rs")).expect("extract types.rs");

        assert!(
            content
                .headings
                .iter()
                .any(|h| h.starts_with("pub struct Fingerprint"))
        );
        assert!(
            content
                .headings
                .iter()
                .any(|h| h.starts_with("pub enum FileType"))
        );
    }

    #[test]
    fn extracts_module_doc_title() {
        let content = extract_rust(Path::new("src/lib.rs")).expect("extract lib.rs");

        assert_eq!(content.title.as_deref(), Some("context-map library"));
    }
}
