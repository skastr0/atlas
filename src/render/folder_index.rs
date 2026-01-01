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
                sig.top_phrases.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
            ));
        }
    }

    // Get files in this folder (direct children only)
    let folder_files: Vec<_> = features
        .iter()
        .filter(|f| f.path.parent() == Some(folder.as_path()))
        .collect();

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
                let (label, counts) = if summary.exported > 0 && !summary.exported_kind_counts.is_empty() {
                    ("Exports", &summary.exported_kind_counts)
                } else {
                    ("Symbols", &summary.kind_counts)
                };

                if !counts.is_empty() {
                    output.push_str(&format!("**{}:** {}\n\n", label, format_symbol_counts(counts)));
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
    let child_folders: std::collections::HashSet<_> = features
        .iter()
        .filter_map(|f| f.path.parent())
        .filter(|p| p.parent() == Some(folder.as_path()))
        .collect();

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
    let folders: std::collections::HashSet<_> = features
        .iter()
        .filter_map(|f| f.path.parent())
        .map(|p| p.to_path_buf())
        .collect();

    for folder in folders {
        let sig = folder_sigs.get(&folder);
        let content = render_folder_index(&folder, features, sig, config);
        indexes.insert(folder, content);
    }

    indexes
}
