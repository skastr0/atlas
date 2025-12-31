//! CONNECTIONS.md and graph generation

use crate::types::{FileFeatures, LinkType};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

const HUB_LIMIT: usize = 10;
const HUB_EDGE_LIMIT: usize = 6;
const MAX_MERMAID_EDGES: usize = 60;
const DEFAULT_EXTENSIONS: [&str; 5] = ["md", "markdown", "txt", "rst", "org"];

#[derive(Debug, Clone)]
pub struct ConnectionGraph {
    /// file path -> list of linked file paths
    pub outbound: HashMap<String, Vec<String>>,
    /// file path -> list of file paths that link to it
    pub inbound: HashMap<String, Vec<String>>,
}

pub struct ConnectionArtifacts {
    pub markdown: String,
    pub mermaid: String,
    pub dot: String,
}

impl ConnectionGraph {
    pub fn from_features(features: &[FileFeatures]) -> Self {
        let lookup = LinkLookup::new(features);
        let mut outbound: HashMap<String, Vec<String>> = HashMap::new();
        let mut inbound: HashMap<String, Vec<String>> = HashMap::new();

        for file in features {
            let source_path = normalize_path(&file.path);
            let mut targets: HashSet<String> = HashSet::new();

            for link in file.links_out.iter().filter(|l| l.link_type == LinkType::Internal) {
                if let Some(resolved) = resolve_internal_link(&link.target, &file.path, &lookup) {
                    if resolved != source_path {
                        targets.insert(resolved);
                    }
                }
            }

            let mut target_list: Vec<String> = targets.into_iter().collect();
            target_list.sort();
            outbound.insert(source_path.clone(), target_list.clone());

            for target in target_list {
                inbound.entry(target).or_default().push(source_path.clone());
            }
        }

        for path in &lookup.paths {
            outbound.entry(path.clone()).or_default();
            inbound.entry(path.clone()).or_default();
        }

        for sources in inbound.values_mut() {
            sources.sort();
            sources.dedup();
        }

        Self { outbound, inbound }
    }

    pub fn hubs(&self, top_n: usize) -> Vec<(String, usize)> {
        let mut counts: Vec<(String, usize)> = self
            .inbound
            .iter()
            .filter(|(_, sources)| !sources.is_empty())
            .map(|(path, sources)| (path.clone(), sources.len()))
            .collect();

        counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        counts.truncate(top_n);
        counts
    }

    pub fn orphans(&self) -> Vec<String> {
        let mut orphans = Vec::new();
        for (path, out) in &self.outbound {
            let inbound_count = self.inbound.get(path).map(|v| v.len()).unwrap_or(0);
            if out.is_empty() && inbound_count == 0 {
                orphans.push(path.clone());
            }
        }
        orphans.sort();
        orphans
    }

    pub fn top_connections(
        &self,
        top_hubs: usize,
        edges_per_hub: usize,
        max_edges: usize,
    ) -> Vec<(String, String)> {
        let mut edges = Vec::new();
        let hubs = self.hubs(top_hubs);

        for (hub, _) in hubs {
            if let Some(sources) = self.inbound.get(&hub) {
                for source in sources.iter().take(edges_per_hub) {
                    edges.push((source.clone(), hub.clone()));
                    if edges.len() >= max_edges {
                        return edges;
                    }
                }
            }
        }

        edges
    }
}

pub fn render_connections(features: &[FileFeatures]) -> ConnectionArtifacts {
    let graph = ConnectionGraph::from_features(features);
    let hubs = graph.hubs(HUB_LIMIT);
    let orphans = graph.orphans();
    let edges = graph.top_connections(HUB_LIMIT, HUB_EDGE_LIMIT, MAX_MERMAID_EDGES);

    let mermaid = render_mermaid(&edges);
    let dot = render_dot(&edges);

    let mut output = String::new();
    output.push_str("# Knowledge Base Connections\n\n");
    output.push_str(
        "_Auto-generated map of internal file connections. Use this to find hubs and orphans._\n\n",
    );

    output.push_str("## Hub Files (Most Referenced)\n\n");
    if hubs.is_empty() {
        output.push_str("_No inbound links detected._\n\n");
    } else {
        for (path, count) in hubs {
            output.push_str(&format!("- **{}** ({} inbound links)\n", path, count));
        }
        output.push('\n');
    }

    output.push_str("## Orphan Files (No Connections)\n\n");
    if orphans.is_empty() {
        output.push_str("_No orphan files detected._\n\n");
    } else {
        for path in &orphans {
            output.push_str(&format!("- {}\n", path));
        }
        output.push('\n');
    }

    output.push_str("## Connection Graph\n\n");
    output.push_str("```mermaid\n");
    output.push_str(&mermaid);
    if !mermaid.ends_with('\n') {
        output.push('\n');
    }
    output.push_str("```\n");

    ConnectionArtifacts {
        markdown: output,
        mermaid,
        dot,
    }
}

struct LinkLookup {
    paths: HashSet<String>,
    filename_map: HashMap<String, Vec<String>>,
    stem_map: HashMap<String, Vec<String>>,
    title_map: HashMap<String, Vec<String>>,
}

impl LinkLookup {
    fn new(features: &[FileFeatures]) -> Self {
        let mut paths = HashSet::new();
        let mut filename_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut stem_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut title_map: HashMap<String, Vec<String>> = HashMap::new();

        for file in features {
            let path_str = normalize_path(&file.path);
            paths.insert(path_str.clone());

            if let Some(filename) = file.path.file_name().and_then(|s| s.to_str()) {
                insert_lookup(&mut filename_map, normalize_key(filename), &path_str);
            }

            if let Some(stem) = file.path.file_stem().and_then(|s| s.to_str()) {
                insert_lookup(&mut stem_map, normalize_key(stem), &path_str);
            }

            if !file.title.is_empty() {
                insert_lookup(&mut title_map, normalize_key(&file.title), &path_str);
            }
        }

        normalize_lookup(&mut filename_map);
        normalize_lookup(&mut stem_map);
        normalize_lookup(&mut title_map);

        Self {
            paths,
            filename_map,
            stem_map,
            title_map,
        }
    }

    fn resolve_key(&self, key: &str) -> Option<String> {
        self.title_map
            .get(key)
            .and_then(|paths| paths.first())
            .cloned()
            .or_else(|| {
                self.stem_map
                    .get(key)
                    .and_then(|paths| paths.first())
                    .cloned()
            })
            .or_else(|| {
                self.filename_map
                    .get(key)
                    .and_then(|paths| paths.first())
                    .cloned()
            })
    }
}

fn insert_lookup(map: &mut HashMap<String, Vec<String>>, key: String, path: &str) {
    map.entry(key).or_default().push(path.to_string());
}

fn normalize_lookup(map: &mut HashMap<String, Vec<String>>) {
    for values in map.values_mut() {
        values.sort();
        values.dedup();
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn normalize_key(value: &str) -> String {
    let lowered = value.trim().to_lowercase();
    let cleaned = lowered.replace('_', " ").replace('-', " ");
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn resolve_internal_link(target: &str, source_path: &Path, lookup: &LinkLookup) -> Option<String> {
    let cleaned = clean_target(target)?;

    if looks_like_path(&cleaned) {
        if let Some(resolved) = resolve_path_target(&cleaned, source_path, lookup) {
            return Some(resolved);
        }
    }

    let key = normalize_key(&cleaned);
    lookup.resolve_key(&key)
}

fn clean_target(target: &str) -> Option<String> {
    let trimmed = target.trim().trim_start_matches('<').trim_end_matches('>');
    if trimmed.is_empty() {
        return None;
    }

    let without_fragment = trimmed.split('#').next().unwrap_or("");
    let without_query = without_fragment.split('?').next().unwrap_or("");
    let cleaned = without_query.trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn looks_like_path(target: &str) -> bool {
    target.starts_with('.') || target.starts_with('/') || target.contains('/') || target.contains('.')
}

fn resolve_path_target(target: &str, source_path: &Path, lookup: &LinkLookup) -> Option<String> {
    let source_dir = source_path.parent().unwrap_or_else(|| Path::new(""));
    let mut candidates = Vec::new();

    if target.starts_with('/') {
        let trimmed = target.trim_start_matches('/');
        candidates.push(normalize_relative_path(Path::new(trimmed)));
    } else {
        candidates.push(normalize_relative_path(&source_dir.join(target)));
        candidates.push(normalize_relative_path(Path::new(target)));
    }

    let mut expanded_candidates = Vec::new();
    for candidate in candidates {
        expanded_candidates.push(candidate.clone());

        if candidate.extension().is_none() {
            for ext in DEFAULT_EXTENSIONS {
                let mut with_ext = candidate.clone();
                with_ext.set_extension(ext);
                expanded_candidates.push(with_ext);
            }
        }
    }

    for candidate in expanded_candidates {
        let normalized = normalize_path(&candidate);
        if lookup.paths.contains(&normalized) {
            return Some(normalized);
        }
    }

    None
}

fn normalize_relative_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir => {}
            Component::Prefix(_) => {}
        }
    }
    normalized
}

fn render_mermaid(edges: &[(String, String)]) -> String {
    let mut output = String::new();
    output.push_str("graph LR\n");

    if edges.is_empty() {
        output.push_str("    %% No connections found\n");
        return output;
    }

    let mut nodes: Vec<String> = edges
        .iter()
        .flat_map(|(from, to)| vec![from.clone(), to.clone()])
        .collect();
    nodes.sort();
    nodes.dedup();

    let node_ids: HashMap<String, String> = nodes
        .iter()
        .enumerate()
        .map(|(idx, path)| (path.clone(), format!("n{}", idx + 1)))
        .collect();

    for (from, to) in edges {
        let from_id = node_ids.get(from).expect("missing from node id");
        let to_id = node_ids.get(to).expect("missing to node id");
        output.push_str(&format!(
            "    {}[\"{}\"] --> {}[\"{}\"]\n",
            from_id,
            escape_mermaid_label(from),
            to_id,
            escape_mermaid_label(to)
        ));
    }

    output
}

fn escape_mermaid_label(label: &str) -> String {
    label.replace('"', "\\\"")
}

fn render_dot(edges: &[(String, String)]) -> String {
    let mut output = String::new();
    output.push_str("digraph Connections {\n");
    output.push_str("    rankdir=LR;\n");

    if edges.is_empty() {
        output.push_str("    // No connections found\n");
        output.push_str("}\n");
        return output;
    }

    for (from, to) in edges {
        output.push_str(&format!("    \"{}\" -> \"{}\";\n", from, to));
    }

    output.push_str("}\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FileFeatures, FileType, KeywordScore, Link, LinkType, PhraseScore, TermScore};
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn make_file(path: &str, title: &str, links: Vec<&str>) -> FileFeatures {
        FileFeatures {
            id: path.to_string(),
            path: PathBuf::from(path),
            file_type: FileType::Markdown,
            title: title.to_string(),
            snippet: String::new(),
            word_count: 0,
            char_count: 0,
            unique_term_count: 0,
            top_terms: Vec::<TermScore>::new(),
            top_phrases: Vec::<PhraseScore>::new(),
            rake_phrases: Vec::<PhraseScore>::new(),
            yake_keywords: Vec::<KeywordScore>::new(),
            links_out: links
                .into_iter()
                .map(|target| Link {
                    target: target.to_string(),
                    link_type: LinkType::Internal,
                })
                .collect(),
            headings: Vec::new(),
            extraction_ok: true,
            extracted_at: 0,
        }
    }

    #[test]
    fn builds_hubs_and_orphans() {
        let features = vec![
            make_file("a.md", "A", vec!["b.md"]),
            make_file("b.md", "B", vec![]),
            make_file("c.md", "C", vec![]),
        ];

        let graph = ConnectionGraph::from_features(&features);
        let hubs = graph.hubs(5);
        assert_eq!(hubs, vec![("b.md".to_string(), 1)]);

        let orphans = graph.orphans();
        assert_eq!(orphans, vec!["c.md".to_string()]);
    }

    #[test]
    fn resolves_relative_and_wiki_links() {
        let features = vec![
            make_file("notes/a.md", "A", vec!["./b.md#section", "C Note", "../root.md"]),
            make_file("notes/b.md", "B", vec![]),
            make_file("c-note.md", "C Note", vec![]),
            make_file("root.md", "Root", vec![]),
        ];

        let graph = ConnectionGraph::from_features(&features);
        let outbound = graph
            .outbound
            .get("notes/a.md")
            .expect("missing outbound for a");

        let outbound_set: HashSet<String> = outbound.iter().cloned().collect();
        let expected: HashSet<String> = vec!["notes/b.md", "c-note.md", "root.md"]
            .into_iter()
            .map(String::from)
            .collect();

        assert_eq!(outbound_set, expected);
    }

    #[test]
    fn falls_back_to_key_lookup_for_dot_wiki_links() {
        let features = vec![
            make_file("notes/index.md", "Index", vec!["v1.0"]),
            make_file("docs/v1.0.md", "v1.0", vec![]),
        ];

        let graph = ConnectionGraph::from_features(&features);
        let outbound = graph
            .outbound
            .get("notes/index.md")
            .expect("missing outbound for index");

        assert_eq!(outbound, &vec!["docs/v1.0.md".to_string()]);
    }
}
