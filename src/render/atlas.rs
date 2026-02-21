//! ROOT_ATLAS.md generation

use crate::config::RenderConfig;
use crate::types::{FileFeatures, FolderSignature};
use std::collections::HashMap;
use std::path::PathBuf;

use super::summarize_code_symbols;

/// Generate ROOT_ATLAS.md content
pub fn render_atlas(
    features: &[FileFeatures],
    folder_sigs: &HashMap<PathBuf, FolderSignature>,
    config: &RenderConfig,
) -> String {
    let mut output = String::new();

    output.push_str("# Knowledge Base Atlas\n\n");
    output.push_str("_Auto-generated map of this knowledge base. Use this to understand what exists before searching._\n\n");

    // Summary stats
    output.push_str("## Overview\n\n");
    output.push_str(&format!("- **Total files:** {}\n", features.len()));

    let total_words: usize = features.iter().map(|f| f.word_count).sum();
    output.push_str(&format!("- **Total words:** {}\n", total_words));

    let folders: std::collections::HashSet<_> =
        features.iter().filter_map(|f| f.path.parent()).collect();
    output.push_str(&format!("- **Folders:** {}\n\n", folders.len()));

    // Folder tree with signatures
    output.push_str("## Folder Structure\n\n");
    let tree = build_folder_tree(features, folder_sigs, config.atlas_folder_depth);
    output.push_str(&tree);
    output.push('\n');

    // Objective slices
    output.push_str(&render_objective_slices(features));

    // Global top terms
    output.push_str("## Top Concepts\n\n");
    let global_terms = aggregate_global_terms(features, 30);
    for (term, _score) in global_terms {
        output.push_str(&format!("- {}\n", term));
    }
    output.push('\n');

    // Navigation hints
    output.push_str("## Navigation\n\n");
    output.push_str("- Each folder has an `INDEX.md` with detailed file listings\n");
    output.push_str("- See `TERMS.md` for concept-to-file mappings\n");
    output.push_str("- File paths are relative to the knowledge base root\n");

    output
}

fn render_objective_slices(features: &[FileFeatures]) -> String {
    let mut output = String::new();

    output.push_str("## Objective Slices\n\n");

    let mut by_size: Vec<&FileFeatures> = features.iter().collect();
    by_size.sort_by(|left, right| {
        right
            .word_count
            .cmp(&left.word_count)
            .then_with(|| left.path.cmp(&right.path))
    });
    output.push_str("### Largest Files\n\n");
    for file in by_size.into_iter().take(10) {
        push_slice_entry(&mut output, file, &file.word_count.to_string());
    }
    output.push('\n');

    let mut by_links: Vec<&FileFeatures> = features.iter().collect();
    by_links.sort_by(|left, right| {
        right
            .links_out
            .len()
            .cmp(&left.links_out.len())
            .then_with(|| left.path.cmp(&right.path))
    });
    output.push_str("### Most Connected\n\n");
    for file in by_links.into_iter().take(10) {
        push_slice_entry(&mut output, file, &file.links_out.len().to_string());
    }
    output.push('\n');

    let mut by_exports: Vec<(&FileFeatures, usize)> = features
        .iter()
        .filter(|f| f.file_type.is_code())
        .map(|f| (f, summarize_code_symbols(&f.headings).exported))
        .filter(|(_, exported)| *exported > 0)
        .collect();
    by_exports.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.path.cmp(&b.0.path)));
    output.push_str("### Most Exported Symbols\n\n");
    if by_exports.is_empty() {
        output.push_str("_No exported code symbols detected._\n\n");
    } else {
        for (file, exported) in by_exports.into_iter().take(10) {
            push_slice_entry(&mut output, file, &exported.to_string());
        }
        output.push('\n');
    }

    let mut by_distinctive: Vec<(&FileFeatures, f32)> =
        features.iter().map(|f| (f, max_term_tfidf(f))).collect();
    by_distinctive.sort_by(|(left_file, left_score), (right_file, right_score)| {
        right_score
            .total_cmp(left_score)
            .then_with(|| left_file.path.cmp(&right_file.path))
    });
    output.push_str("### Most Distinctive\n\n");
    for (file, score) in by_distinctive.into_iter().take(10) {
        push_slice_entry(&mut output, file, &format!("{:.3}", score));
    }
    output.push('\n');

    let mut by_diverse: Vec<&FileFeatures> = features.iter().collect();
    by_diverse.sort_by(|left, right| {
        right
            .unique_term_count
            .cmp(&left.unique_term_count)
            .then_with(|| left.path.cmp(&right.path))
    });
    output.push_str("### Most Diverse\n\n");
    for file in by_diverse.into_iter().take(10) {
        push_slice_entry(&mut output, file, &file.unique_term_count.to_string());
    }
    output.push('\n');

    output
}

fn push_slice_entry(output: &mut String, file: &FileFeatures, metric: &str) {
    output.push_str(&format!(
        "- **{}** ({}) - {}\n",
        file.title,
        metric,
        file.path.display()
    ));
}

fn max_term_tfidf(file: &FileFeatures) -> f32 {
    file.top_terms
        .iter()
        .map(|term| term.tfidf)
        .fold(0.0, f32::max)
}

fn build_folder_tree(
    features: &[FileFeatures],
    folder_sigs: &HashMap<PathBuf, FolderSignature>,
    max_depth: usize,
) -> String {
    let mut output = String::new();

    // Get unique folders
    let mut folders: Vec<_> = features
        .iter()
        .filter_map(|f| f.path.parent())
        .map(|p| p.to_path_buf())
        .collect();
    folders.sort();
    folders.dedup();

    // Filter by depth
    let folders: Vec<_> = folders
        .into_iter()
        .filter(|p| p.components().count() <= max_depth)
        .collect();

    for folder in folders {
        let depth = folder.components().count();
        let indent = "  ".repeat(depth);

        let sig = folder_sigs.get(&folder);
        let file_count = sig.map(|s| s.file_count).unwrap_or(0);
        let top_phrases = sig
            .map(|s| s.top_phrases.iter().take(3).cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        let folder_name = folder
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "(root)".to_string());

        if top_phrases.is_empty() {
            output.push_str(&format!(
                "{}- **{}/** ({} files)\n",
                indent, folder_name, file_count
            ));
        } else {
            output.push_str(&format!(
                "{}- **{}/** ({} files) — {}\n",
                indent,
                folder_name,
                file_count,
                top_phrases.join(", ")
            ));
        }
    }

    output
}

fn aggregate_global_terms(features: &[FileFeatures], top_n: usize) -> Vec<(String, f32)> {
    let mut term_scores: HashMap<String, f32> = HashMap::new();

    for file in features {
        for term in &file.top_terms {
            *term_scores.entry(term.term.clone()).or_insert(0.0) += term.tfidf;
        }
    }

    let mut sorted: Vec<_> = term_scores.into_iter().collect();
    sorted.sort_by(|(left_term, left_score), (right_term, right_score)| {
        right_score
            .total_cmp(left_score)
            .then_with(|| left_term.cmp(right_term))
    });
    sorted.truncate(top_n);
    sorted
}

#[cfg(test)]
mod tests {
    use super::render_atlas;
    use crate::config::RenderConfig;
    use crate::types::{FileFeatures, FileType, FolderSignature, Link, LinkType, TermScore};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_feature(
        id: &str,
        path: &str,
        title: &str,
        word_count: usize,
        links: usize,
        unique_term_count: usize,
        top_terms: Vec<(&str, f32)>,
    ) -> FileFeatures {
        FileFeatures {
            id: id.to_string(),
            path: PathBuf::from(path),
            file_type: FileType::Markdown,
            title: title.to_string(),
            snippet: String::new(),
            word_count,
            char_count: word_count,
            unique_term_count,
            top_terms: top_terms
                .into_iter()
                .map(|(term, tfidf)| TermScore {
                    term: term.to_string(),
                    tf: tfidf,
                    tfidf,
                })
                .collect(),
            top_phrases: Vec::new(),
            rake_phrases: Vec::new(),
            yake_keywords: Vec::new(),
            links_out: (0..links)
                .map(|idx| Link {
                    target: format!("https://example.com/{id}/{idx}"),
                    link_type: LinkType::External,
                })
                .collect(),
            headings: Vec::new(),
            extraction_ok: true,
            extracted_at: 0,
        }
    }

    fn assert_order(content: &str, first: &str, second: &str) {
        let first_idx = content.find(first).expect("first marker missing");
        let second_idx = content.find(second).expect("second marker missing");

        assert!(
            first_idx < second_idx,
            "expected `{first}` before `{second}` in:\n{content}"
        );
    }

    fn between<'a>(content: &'a str, start: &str, end: &str) -> &'a str {
        let start_idx = content.find(start).expect("start marker missing");
        let start_slice = &content[start_idx + start.len()..];
        let end_idx = start_slice.find(end).expect("end marker missing");
        &start_slice[..end_idx]
    }

    #[test]
    fn renders_objective_slices_and_top_concepts_deterministically_for_ties() {
        let first = make_feature(
            "doc-z",
            "docs/z.md",
            "Doc Z",
            100,
            2,
            7,
            vec![("zeta", 1.0)],
        );
        let second = make_feature(
            "doc-a",
            "docs/a.md",
            "Doc A",
            100,
            2,
            7,
            vec![("alpha", 1.0)],
        );

        let folder_sigs: HashMap<PathBuf, FolderSignature> = HashMap::new();
        let config = RenderConfig::default();

        let forward = render_atlas(&[first.clone(), second.clone()], &folder_sigs, &config);
        let reversed = render_atlas(&[second, first], &folder_sigs, &config);

        assert_eq!(forward, reversed);

        let largest = between(&forward, "### Largest Files\n\n", "### Most Connected\n\n");
        assert_order(
            largest,
            "- **Doc A** (100) - docs/a.md\n",
            "- **Doc Z** (100) - docs/z.md\n",
        );

        let connected = between(
            &forward,
            "### Most Connected\n\n",
            "### Most Exported Symbols\n\n",
        );
        assert_order(
            connected,
            "- **Doc A** (2) - docs/a.md\n",
            "- **Doc Z** (2) - docs/z.md\n",
        );

        let distinctive = between(&forward, "### Most Distinctive\n\n", "### Most Diverse\n\n");
        assert_order(
            distinctive,
            "- **Doc A** (1.000) - docs/a.md\n",
            "- **Doc Z** (1.000) - docs/z.md\n",
        );

        let diverse = between(&forward, "### Most Diverse\n\n", "## Top Concepts\n\n");
        assert_order(
            diverse,
            "- **Doc A** (7) - docs/a.md\n",
            "- **Doc Z** (7) - docs/z.md\n",
        );

        let concepts = between(&forward, "## Top Concepts\n\n", "## Navigation\n\n");
        assert_order(concepts, "- alpha\n", "- zeta\n");
    }
}
