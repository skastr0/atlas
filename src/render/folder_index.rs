//! Per-folder INDEX.md generation

use crate::config::RenderConfig;
use crate::types::{FileFeatures, FolderSignature};
use std::collections::HashMap;
use std::path::PathBuf;

use super::{format_symbol_counts, summarize_code_symbols};

/// Generate INDEX.md content for a folder
pub fn render_folder_index(
    folder: &PathBuf,
    features: &[FileFeatures],
    folder_sig: Option<&FolderSignature>,
    config: &RenderConfig,
) -> String {
    let mut output = String::new();

    let folder_name = folder
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "(root)".to_string());

    output.push_str(&format!("# {}\n\n", folder_name));

    // Folder signature summary
    if let Some(sig) = folder_sig {
        if !sig.top_phrases.is_empty() {
            output.push_str(&format!(
                "_Key topics: {}_\n\n",
                sig.top_phrases
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    // Get files in this folder (direct children only)
    let mut folder_files: Vec<_> = features
        .iter()
        .filter(|f| f.path.parent() == Some(folder.as_path()))
        .collect();
    folder_files.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.title.cmp(&right.title))
    });

    if folder_files.is_empty() {
        output.push_str("_No indexed files in this folder._\n");
        return output;
    }

    let max_files = config.atlas_max_files_per_folder;
    let (display_files, omitted_count) = if max_files == 0 || folder_files.len() <= max_files {
        (folder_files.as_slice(), 0)
    } else {
        (&folder_files[..max_files], folder_files.len() - max_files)
    };

    // Files section
    output.push_str("## Files\n\n");

    for file in display_files {
        let filename = file
            .path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        output.push_str(&format!("### {}\n\n", file.title));
        output.push_str(&format!("`{}`\n\n", filename));

        let mut rendered_code_summary = false;
        if file.file_type.is_code() {
            let summary = summarize_code_symbols(&file.headings);
            if summary.total > 0 {
                let (label, counts) =
                    if summary.exported > 0 && !summary.exported_kind_counts.is_empty() {
                        ("Exports", &summary.exported_kind_counts)
                    } else {
                        ("Symbols", &summary.kind_counts)
                    };

                if !counts.is_empty() {
                    output.push_str(&format!(
                        "**{}:** {}\n\n",
                        label,
                        format_symbol_counts(counts)
                    ));
                }

                let symbols = if summary.exported > 0 && !summary.top_exported_symbols.is_empty() {
                    &summary.top_exported_symbols
                } else {
                    &summary.top_symbols
                };

                if !symbols.is_empty() {
                    let list = symbols
                        .iter()
                        .take(5)
                        .map(|symbol| format!("`{}`", symbol))
                        .collect::<Vec<_>>()
                        .join(", ");
                    output.push_str(&format!("**Top symbols:** {}\n\n", list));
                }

                rendered_code_summary = true;
            }
        }

        if !rendered_code_summary {
            // Snippet
            if !file.snippet.is_empty() {
                output.push_str(&format!("{}\n\n", file.snippet));
            }

            // Top phrases
            if !file.top_phrases.is_empty() {
                let phrases: Vec<_> = file
                    .top_phrases
                    .iter()
                    .take(5)
                    .map(|p| p.phrase.as_str())
                    .collect();
                output.push_str(&format!("**Phrases:** {}\n\n", phrases.join(", ")));
            }

            // Top terms
            if !file.top_terms.is_empty() {
                let terms: Vec<_> = file
                    .top_terms
                    .iter()
                    .take(5)
                    .map(|t| t.term.as_str())
                    .collect();
                output.push_str(&format!("**Terms:** {}\n\n", terms.join(", ")));
            }
        }

        // Stats
        output.push_str(&format!(
            "_{} words, {} chars_\n\n",
            file.word_count, file.char_count
        ));

        output.push_str("---\n\n");
    }

    if omitted_count > 0 {
        output.push_str(&format!(
            "_Showing first {} of {} files. Increase atlas_max_files_per_folder to show more._\n\n",
            display_files.len(),
            folder_files.len()
        ));
    }

    // Child folders
    let mut child_folders: Vec<_> = features
        .iter()
        .filter_map(|f| f.path.parent())
        .filter(|p| p.parent() == Some(folder.as_path()))
        .map(|p| p.to_path_buf())
        .collect();
    child_folders.sort();
    child_folders.dedup();

    if !child_folders.is_empty() {
        output.push_str("## Subfolders\n\n");
        for child in child_folders {
            let name = child
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            output.push_str(&format!("- [{}](./{})\n", name, name));
        }
    }

    output
}

/// Generate all folder indexes
pub fn render_all_folder_indexes(
    features: &[FileFeatures],
    folder_sigs: &HashMap<PathBuf, FolderSignature>,
    config: &RenderConfig,
) -> HashMap<PathBuf, String> {
    let mut indexes = HashMap::new();

    // Get all unique folders
    let mut folders: Vec<_> = features
        .iter()
        .filter_map(|f| f.path.parent())
        .map(|p| p.to_path_buf())
        .collect();
    folders.sort();
    folders.dedup();

    for folder in folders {
        let sig = folder_sigs.get(&folder);
        let content = render_folder_index(&folder, features, sig, config);
        indexes.insert(folder, content);
    }

    indexes
}

#[cfg(test)]
mod tests {
    use super::{render_all_folder_indexes, render_folder_index};
    use crate::config::RenderConfig;
    use crate::types::{FileFeatures, FileType, Link, LinkType};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_feature(id: &str, path: &str, title: &str) -> FileFeatures {
        FileFeatures {
            id: id.to_string(),
            path: PathBuf::from(path),
            file_type: FileType::Markdown,
            title: title.to_string(),
            snippet: String::new(),
            word_count: 0,
            char_count: 0,
            unique_term_count: 0,
            top_terms: Vec::new(),
            top_phrases: Vec::new(),
            rake_phrases: Vec::new(),
            yake_keywords: Vec::new(),
            links_out: vec![Link {
                target: "https://example.com".to_string(),
                link_type: LinkType::External,
            }],
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

    #[test]
    fn sorts_files_before_truncation_and_is_stable_for_shuffled_input() {
        let config = RenderConfig {
            atlas_folder_depth: 3,
            atlas_max_files_per_folder: 1,
        };
        let folder = PathBuf::from("docs");

        let first = make_feature("doc-b", "docs/b.md", "B Title");
        let second = make_feature("doc-a", "docs/a.md", "A Title");

        let forward = render_folder_index(&folder, &[first.clone(), second.clone()], None, &config);
        let reversed = render_folder_index(&folder, &[second, first], None, &config);

        assert_eq!(forward, reversed);
        assert!(forward.contains("`a.md`"));
        assert!(!forward.contains("`b.md`"));
        assert!(forward.contains("_Showing first 1 of 2 files."));
    }

    #[test]
    fn sorts_subfolders_alphabetically_before_rendering() {
        let config = RenderConfig::default();
        let folder = PathBuf::from("docs");
        let features = vec![
            make_feature("root", "docs/root.md", "Root"),
            make_feature("z-child", "docs/zeta/file.md", "Z child"),
            make_feature("a-child", "docs/alpha/file.md", "A child"),
        ];

        let rendered = render_folder_index(&folder, &features, None, &config);

        assert_order(&rendered, "- [alpha](./alpha)", "- [zeta](./zeta)");
    }

    #[test]
    fn render_all_folder_indexes_is_stable_for_shuffled_input() {
        let config = RenderConfig::default();
        let folder_sigs = HashMap::new();

        let first = make_feature("doc-b", "docs/b.md", "B Title");
        let second = make_feature("doc-a", "docs/a.md", "A Title");
        let third = make_feature("doc-n", "notes/n.md", "N Title");

        let forward = render_all_folder_indexes(
            &[first.clone(), second.clone(), third.clone()],
            &folder_sigs,
            &config,
        );
        let reversed = render_all_folder_indexes(&[third, second, first], &folder_sigs, &config);

        assert_eq!(forward, reversed);
    }
}
