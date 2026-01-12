//! TypeScript and TSX extraction using tree-sitter

use super::treesitter::{parse, Language};
use super::ExtractedContent;
use anyhow::Result;
use std::fs;
use std::path::Path;
use tree_sitter::Node;

/// Extract content from a TypeScript or TSX file.
pub fn extract_typescript(path: &Path, is_tsx: bool) -> Result<ExtractedContent> {
    let source = fs::read_to_string(path)?;
    let language = if is_tsx {
        Language::Tsx
    } else {
        Language::TypeScript
    };

    let mut headings = Vec::new();
    let mut links = Vec::new();
    let mut doc_comments = Vec::new();
    let mut success = true;

    if let Some(tree) = parse(&source, language) {
        let root = tree.root_node();
        collect_nodes(root, &source, &mut headings, &mut links);
        collect_doc_comments(root, &source, &mut doc_comments);
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
        title: None,
        headings,
        links,
        success,
    })
}

fn collect_nodes(node: Node, source: &str, headings: &mut Vec<String>, links: &mut Vec<String>) {
    match node.kind() {
        "function_declaration" => add_heading(node, "function", source, headings),
        "class_declaration" => add_heading(node, "class", source, headings),
        "interface_declaration" => add_heading(node, "interface", source, headings),
        "type_alias_declaration" => add_heading(node, "type", source, headings),
        "enum_declaration" => add_heading(node, "enum", source, headings),
        "import_statement" => {
            if let Some(link) = extract_import_source(node, source) {
                links.push(link);
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_nodes(child, source, headings, links);
    }
}

fn collect_doc_comments(node: Node, source: &str, docs: &mut Vec<String>) {
    if node.kind() == "comment" {
        if let Some(text) = node_text(node, source) {
            let trimmed = text.trim_start();
            if is_jsdoc_comment(trimmed) {
                let normalized = normalize_doc_comment(trimmed);
                if !normalized.is_empty() {
                    docs.push(normalized);
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_doc_comments(child, source, docs);
    }
}

fn is_jsdoc_comment(text: &str) -> bool {
    let trimmed = text.trim_start();
    if !trimmed.starts_with("/**") {
        return false;
    }
    if trimmed.starts_with("/***") || trimmed.starts_with("/**/") {
        return false;
    }
    true
}

fn normalize_doc_comment(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.starts_with("/**") {
        let without_start = trimmed.trim_start_matches("/**");
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

fn add_heading(node: Node, kind_label: &str, source: &str, headings: &mut Vec<String>) {
    let Some(name) = extract_declaration_name(node, source) else {
        return;
    };
    let export_prefix = if is_exported(node, source) {
        "export "
    } else {
        ""
    };
    headings.push(format!("{export_prefix}{kind_label} {name}"));
}

fn extract_declaration_name(node: Node, source: &str) -> Option<String> {
    if let Some(name_node) = node.child_by_field_name("name") {
        return node_text(name_node, source).map(|name| name.to_string());
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" | "type_identifier" => {
                return node_text(child, source).map(|name| name.to_string());
            }
            _ => {}
        }
    }

    None
}

fn is_exported(node: Node, source: &str) -> bool {
    if has_export_ancestor(node) {
        return true;
    }

    node_has_export_modifier(node, source)
}

fn has_export_ancestor(node: Node) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "export_statement" {
            return true;
        }
        current = parent.parent();
    }
    false
}

fn node_has_export_modifier(node: Node, source: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "export" || child.kind() == "export_keyword" {
            return true;
        }
        if child.kind() == "modifier" || child.kind() == "modifiers" {
            if node_text(child, source)
                .map(|text| text.contains("export"))
                .unwrap_or(false)
            {
                return true;
            }
        }
    }
    false
}

fn extract_import_source(node: Node, source: &str) -> Option<String> {
    if let Some(source_node) = node.child_by_field_name("source") {
        if let Some(value) = string_literal_value(source_node, source) {
            return Some(value);
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "string" {
            if let Some(value) = string_literal_value(child, source) {
                return Some(value);
            }
        }
    }

    None
}

fn string_literal_value(node: Node, source: &str) -> Option<String> {
    let raw = node_text(node, source)?.trim();
    let unquoted = raw.trim_matches(&['"', '\'', '`'][..]).trim().to_string();

    if unquoted.is_empty() {
        None
    } else {
        Some(unquoted)
    }
}

fn node_text<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    source.get(node.start_byte()..node.end_byte())
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::extract_typescript;
    use anyhow::Result;
    use std::fs;

    #[test]
    fn extracts_symbols_and_imports_from_typescript() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("sample.ts");
        let source = r#"
import React from "react";
import { Foo } from './foo';

function internal() {}
export function exported() {}
export class Widget {}
interface Shape {}
type Alias = string;
export enum Status { Active }
"#;
        fs::write(&path, source)?;

        let content = extract_typescript(&path, false)?;

        assert!(content.success);
        assert!(content.headings.contains(&"function internal".to_string()));
        assert!(content
            .headings
            .contains(&"export function exported".to_string()));
        assert!(content
            .headings
            .contains(&"export class Widget".to_string()));
        assert!(content.headings.contains(&"interface Shape".to_string()));
        assert!(content.headings.contains(&"type Alias".to_string()));
        assert!(content.headings.contains(&"export enum Status".to_string()));
        assert!(content.links.contains(&"react".to_string()));
        assert!(content.links.contains(&"./foo".to_string()));

        Ok(())
    }

    #[test]
    fn extracts_symbols_from_tsx() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("sample.tsx");
        let source = r#"
import { useState } from "react";

export function Button() {
  return <div />;
}
"#;
        fs::write(&path, source)?;

        let content = extract_typescript(&path, true)?;

        assert!(content.success);
        assert!(content
            .headings
            .contains(&"export function Button".to_string()));
        assert!(content.links.contains(&"react".to_string()));

        Ok(())
    }

    #[test]
    fn extracts_doc_comments_from_typescript() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("docs.ts");
        let source = r#"
/** First doc. */
export function first() {}

/* not docs */
const value = 1;

/**
 * Second doc
 * next line
 */
export function second() {}
"#;
        fs::write(&path, source)?;

        let content = extract_typescript(&path, false)?;

        assert_eq!(content.text, "First doc.\n\nSecond doc next line");

        Ok(())
    }

    #[test]
    fn extracts_doc_comments_from_tsx() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("docs.tsx");
        let source = r#"
/** Button docs */
export function Button() {
  return <div />;
}
"#;
        fs::write(&path, source)?;

        let content = extract_typescript(&path, true)?;

        assert_eq!(content.text, "Button docs");

        Ok(())
    }

    #[test]
    fn falls_back_to_headings_when_docs_missing() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("no_docs.ts");
        let source = "export class Example {}";
        fs::write(&path, source)?;

        let content = extract_typescript(&path, false)?;

        assert_eq!(content.text, "export class Example");

        Ok(())
    }
}
