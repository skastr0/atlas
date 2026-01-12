//! Rust source extraction using tree-sitter

use super::treesitter::{parse, Language};
use super::ExtractedContent;
use anyhow::Result;
use std::fs;
use std::path::Path;
use tree_sitter::Node;

/// Extract content from Rust source files.
pub fn extract_rust(path: &Path) -> Result<ExtractedContent> {
    let source = fs::read_to_string(path)?;
    let title = extract_module_doc_title(&source).or_else(|| {
        source
            .lines()
            .find(|line| !line.trim().is_empty())
            .filter(|line| line.len() < 200)
            .map(|line| line.trim().to_string())
    });

    let mut headings = Vec::new();
    let mut links = Vec::new();
    let mut doc_comments = Vec::new();
    let mut success = true;

    if let Some(tree) = parse(&source, Language::Rust) {
        let root = tree.root_node();
        collect_nodes(&source, root, &mut headings, &mut links);
        collect_doc_comments(&source, root, &mut doc_comments);
    } else {
        success = false;
    }

    let text = if !doc_comments.is_empty() {
        doc_comments.join("\n\n")
    } else {
        headings.join(", ")
    };

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

fn collect_doc_comments(source: &str, node: Node, docs: &mut Vec<String>) {
    match node.kind() {
        "line_comment" => {
            if let Some(text) = node_text(source, node) {
                let trimmed = text.trim_start();
                if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                    let normalized = normalize_doc_comment(trimmed);
                    if !normalized.is_empty() {
                        docs.push(normalized);
                    }
                }
            }
        }
        "block_comment" => {
            if let Some(text) = node_text(source, node) {
                let trimmed = text.trim_start();
                if trimmed.starts_with("/**") || trimmed.starts_with("/*!") {
                    let normalized = normalize_doc_comment(trimmed);
                    if !normalized.is_empty() {
                        docs.push(normalized);
                    }
                }
            }
        }
        "attribute_item" => {
            if let Some(doc) = extract_doc_attribute(source, node) {
                docs.push(doc);
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_doc_comments(source, child, docs);
    }
}

fn normalize_doc_comment(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.starts_with("///") {
        return normalize_whitespace(trimmed.trim_start_matches("///"));
    }
    if trimmed.starts_with("//!") {
        return normalize_whitespace(trimmed.trim_start_matches("//!"));
    }
    if trimmed.starts_with("/**") || trimmed.starts_with("/*!") {
        let without_start = if trimmed.starts_with("/**") {
            trimmed.trim_start_matches("/**")
        } else {
            trimmed.trim_start_matches("/*!")
        };
        let without_end = without_start.trim_end_matches("*/");
        let cleaned = without_end
            .lines()
            .map(|line| {
                let line_trimmed = line.trim();
                let line_trimmed = line_trimmed.strip_prefix('*').unwrap_or(line_trimmed);
                line_trimmed.trim()
            })
            .collect::<Vec<_>>()
            .join(" ");
        return normalize_whitespace(&cleaned);
    }

    normalize_whitespace(trimmed)
}

fn extract_doc_attribute<'a>(source: &str, node: Node<'a>) -> Option<String> {
    let path_text = node
        .child_by_field_name("path")
        .and_then(|path_node| node_text(source, path_node))
        .map(|text| text.trim());

    let is_doc = match path_text {
        Some("doc") => true,
        Some(_) => false,
        None => node_text(source, node).map_or(false, |text| {
            let trimmed = text.trim_start();
            trimmed.starts_with("#[doc") || trimmed.starts_with("#![doc")
        }),
    };

    if !is_doc {
        return None;
    }

    let value_node = find_descendant(node, &["string_literal", "raw_string_literal"])?;
    let raw_value = node_text(source, value_node)?;
    let normalized = normalize_whitespace(&normalize_doc_attribute_value(raw_value));
    if normalized.is_empty() {
        return None;
    }
    Some(normalized)
}

fn normalize_doc_attribute_value(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.starts_with('r') {
        if let (Some(start), Some(end)) = (trimmed.find('"'), trimmed.rfind('"')) {
            if end > start {
                return trimmed[start + 1..end].to_string();
            }
        }
    }
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed.to_string()
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
    collect_descendants(
        node,
        &["type_identifier", "scoped_type_identifier"],
        &mut type_nodes,
    );

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
    normalized = normalized
        .trim_end()
        .trim_end_matches('{')
        .trim_end()
        .to_string();
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
    use std::fs;
    use std::path::Path;
    use tempfile::NamedTempFile;

    #[test]
    fn extracts_symbols_and_use_links_from_main() {
        let content = extract_rust(Path::new("src/main.rs")).expect("extract main.rs");

        assert!(content.headings.iter().any(|h| h.starts_with("fn main")));
        assert!(content.headings.iter().any(|h| h.starts_with("struct Cli")));
        assert!(content
            .headings
            .iter()
            .any(|h| h.starts_with("enum Commands")));
        assert!(content
            .headings
            .iter()
            .any(|h| h.starts_with("pub enum LogLevel")));
        assert!(content
            .headings
            .iter()
            .any(|h| h.starts_with("impl LogLevel")));
        assert!(content
            .links
            .iter()
            .any(|link| link.contains("anyhow::Result")));
    }

    #[test]
    fn extracts_public_symbols_from_types() {
        let content = extract_rust(Path::new("src/types.rs")).expect("extract types.rs");

        assert!(content
            .headings
            .iter()
            .any(|h| h.starts_with("pub struct Fingerprint")));
        assert!(content
            .headings
            .iter()
            .any(|h| h.starts_with("pub enum FileType")));
    }

    #[test]
    fn extracts_module_doc_title() {
        let content = extract_rust(Path::new("src/lib.rs")).expect("extract lib.rs");

        assert_eq!(content.title.as_deref(), Some("context-map library"));
    }

    #[test]
    fn extracts_doc_comments_as_text() {
        let source = r#"
//! Module docs
/// Item docs.
#[doc = "Attribute docs"]
/*!
 * Block docs
 * second line
 */
pub fn demo() {}
"#;
        let file = NamedTempFile::new().expect("tempfile");
        fs::write(file.path(), source).expect("write temp source");

        let content = extract_rust(file.path()).expect("extract temp source");

        assert_eq!(
            content.text,
            "Module docs\n\nItem docs.\n\nAttribute docs\n\nBlock docs second line"
        );
    }

    #[test]
    fn falls_back_to_headings_when_docs_missing() {
        let source = "pub struct Example {}";
        let file = NamedTempFile::new().expect("tempfile");
        fs::write(file.path(), source).expect("write temp source");

        let content = extract_rust(file.path()).expect("extract temp source");

        assert_eq!(content.text, "pub struct Example");
    }
}
