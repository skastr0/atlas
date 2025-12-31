//! Markdown text extraction

use super::ExtractedContent;
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Extract content from Markdown file
pub fn extract_markdown(path: &Path) -> Result<ExtractedContent> {
    let text = fs::read_to_string(path)?;

    // Extract title (first # heading)
    let title = extract_title(&text);

    // Extract all headings
    let headings = extract_headings(&text);

    // Extract links
    let links = extract_links(&text);

    Ok(ExtractedContent {
        text,
        title,
        headings,
        links,
        success: true,
    })
}

fn extract_title(text: &str) -> Option<String> {
    // Match first # heading (not ##)
    let re = Regex::new(r"^#\s+(.+)$").ok()?;
    for line in text.lines() {
        if let Some(caps) = re.captures(line) {
            return Some(caps.get(1)?.as_str().to_string());
        }
    }
    None
}

fn extract_headings(text: &str) -> Vec<String> {
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

fn extract_links(text: &str) -> Vec<String> {
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

    links
}
