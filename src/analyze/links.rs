//! Link and citation extraction

use crate::types::{Link, LinkType};
use regex::Regex;

/// Extract all links from text
pub fn extract_links(text: &str) -> Vec<Link> {
    let mut links = Vec::new();

    // URLs (http/https)
    let url_re = Regex::new(r"https?://[^\s\)\]>]+").unwrap();
    for m in url_re.find_iter(text) {
        links.push(Link {
            target: m.as_str().to_string(),
            link_type: LinkType::External,
        });
    }

    // DOIs
    let doi_re = Regex::new(r"(?i)(?:doi:?\s*)?10\.\d{4,}/[^\s]+").unwrap();
    for m in doi_re.find_iter(text) {
        links.push(Link {
            target: m.as_str().to_string(),
            link_type: LinkType::Citation,
        });
    }

    // arXiv IDs
    let arxiv_re = Regex::new(r"(?i)arxiv:?\s*\d{4}\.\d{4,5}(?:v\d+)?").unwrap();
    for m in arxiv_re.find_iter(text) {
        links.push(Link {
            target: m.as_str().to_string(),
            link_type: LinkType::Citation,
        });
    }

    // ISBNs
    let isbn_re = Regex::new(r"(?i)isbn[:\s-]*(?:\d[\d\s-]{8,}[\dxX])").unwrap();
    for m in isbn_re.find_iter(text) {
        links.push(Link {
            target: m.as_str().to_string(),
            link_type: LinkType::Citation,
        });
    }

    // Markdown links [text](path) - internal if relative path
    let md_link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    for caps in md_link_re.captures_iter(text) {
        if let Some(url) = caps.get(2) {
            let target = url.as_str();
            // Skip if already captured as URL
            if !target.starts_with("http") {
                links.push(Link {
                    target: target.to_string(),
                    link_type: LinkType::Internal,
                });
            }
        }
    }

    // Wiki-style links [[link]]
    let wiki_re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
    for caps in wiki_re.captures_iter(text) {
        if let Some(link) = caps.get(1) {
            links.push(Link {
                target: link.as_str().to_string(),
                link_type: LinkType::Internal,
            });
        }
    }

    links
}

/// Count links by type
pub fn count_links_by_type(links: &[Link]) -> (usize, usize, usize) {
    let mut internal = 0;
    let mut external = 0;
    let mut citations = 0;

    for link in links {
        match link.link_type {
            LinkType::Internal => internal += 1,
            LinkType::External => external += 1,
            LinkType::Citation => citations += 1,
        }
    }

    (internal, external, citations)
}
