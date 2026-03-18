//! Tantivy backend for feature cache

use crate::types::FileFeatures;
use anyhow::{bail, Context, Result};
use std::path::{Component, Path, PathBuf};
use tantivy::schema::{Schema, Value, STORED, STRING, TEXT};
use tantivy::{Index, IndexWriter, TantivyDocument, Term};

pub const INDEX_DIR_NAME: &str = "tantivy-v2";
pub const SEARCH_INDEX_VERSION: &str = INDEX_DIR_NAME;

const REQUIRED_FIELDS: [&str; 10] = [
    "id",
    "path",
    "path_text",
    "scope_terms",
    "title",
    "snippet",
    "body",
    "file_type",
    "extension",
    "features",
];

pub struct PreparedIndex {
    pub index: Index,
    pub needs_reindex: bool,
}

pub fn index_dir(cmap_path: &Path) -> PathBuf {
    cmap_path.join("index").join(INDEX_DIR_NAME)
}

pub fn prepare_index(index_dir: &Path) -> Result<PreparedIndex> {
    if !index_dir.exists() {
        std::fs::create_dir_all(index_dir).context("Failed to create tantivy index directory")?;
    }

    let schema = build_schema();
    if !index_dir.join("meta.json").exists() {
        let index =
            Index::create_in_dir(index_dir, schema).context("Failed to create tantivy index")?;
        return Ok(PreparedIndex {
            index,
            needs_reindex: true,
        });
    }

    let index = Index::open_in_dir(index_dir).context("Failed to open existing tantivy index")?;
    if has_compatible_schema(&index.schema()) {
        return Ok(PreparedIndex {
            index,
            needs_reindex: false,
        });
    }

    std::fs::remove_dir_all(index_dir).context("Failed to reset incompatible tantivy index")?;
    std::fs::create_dir_all(index_dir).context("Failed to recreate tantivy index directory")?;
    let index = Index::create_in_dir(index_dir, build_schema())
        .context("Failed to create tantivy index")?;

    Ok(PreparedIndex {
        index,
        needs_reindex: true,
    })
}

pub fn open_index(index_dir: &Path) -> Result<Index> {
    if !index_dir.join("meta.json").exists() {
        bail!(
            "Search index {} not found. Run `cmap build` first.",
            SEARCH_INDEX_VERSION
        );
    }

    let index = Index::open_in_dir(index_dir).context("Failed to open tantivy index")?;
    if !has_compatible_schema(&index.schema()) {
        bail!(
            "Search index {} is incompatible with this build. Run `cmap build` first.",
            SEARCH_INDEX_VERSION
        );
    }

    Ok(index)
}

/// Delete features by their ID
pub fn delete_documents(index: &Index, writer: &mut IndexWriter, ids: &[String]) -> Result<()> {
    let schema = index.schema();
    let id_field = schema.get_field("id").unwrap();

    for id in ids {
        writer.delete_term(Term::from_field_text(id_field, id));
    }

    Ok(())
}

/// Add features to the index
pub fn add_documents(
    index: &Index,
    writer: &mut IndexWriter,
    features_list: &[(FileFeatures, String)],
) -> Result<()> {
    let schema = index.schema();
    let id_field = schema.get_field("id").unwrap();
    let path_field = schema.get_field("path").unwrap();
    let path_text_field = schema.get_field("path_text").unwrap();
    let scope_terms_field = schema.get_field("scope_terms").unwrap();
    let title_field = schema.get_field("title").unwrap();
    let snippet_field = schema.get_field("snippet").unwrap();
    let body_field = schema.get_field("body").unwrap();
    let file_type_field = schema.get_field("file_type").unwrap();
    let extension_field = schema.get_field("extension").unwrap();
    let features_field = schema.get_field("features").unwrap();

    for (features, body) in features_list {
        let serialized = serde_json::to_vec(features).context("Failed to serialize features")?;
        let file_type_str = features.file_type.search_term();
        let extension = normalized_extension(&features.path);
        let path_text = searchable_path_text(&features.path);
        let scope_terms = normalized_scope_terms(&features.path);

        let mut document = TantivyDocument::default();
        document.add_text(id_field, features.id.clone());
        document.add_text(path_field, features.path.to_string_lossy().into_owned());
        document.add_text(path_text_field, path_text);
        document.add_text(title_field, features.title.clone());
        document.add_text(snippet_field, features.snippet.clone());
        document.add_text(body_field, body.clone());
        document.add_text(file_type_field, file_type_str.to_string());
        document.add_text(extension_field, extension);
        document.add_bytes(features_field, &serialized);
        for scope_term in scope_terms {
            document.add_text(scope_terms_field, scope_term);
        }

        writer
            .add_document(document)
            .context("Failed to add document to tantivy")?;
    }

    Ok(())
}

/// Load all cached features from the index
pub fn load_all_features(index: &Index) -> Result<Vec<FileFeatures>> {
    let searcher = index.reader()?.searcher();
    let schema = index.schema();
    let features_field = schema.get_field("features").unwrap();

    let mut all_features = Vec::new();

    for segment_reader in searcher.segment_readers() {
        let store_reader = segment_reader.get_store_reader(10)?;
        for doc_id in segment_reader.doc_ids_alive() {
            let doc: TantivyDocument = store_reader.get(doc_id)?;
            if let Some(field_value) = doc.get_first(features_field) {
                if let Some(bytes) = field_value.as_bytes() {
                    if let Ok(features) = serde_json::from_slice::<FileFeatures>(bytes) {
                        all_features.push(features);
                    }
                }
            }
        }
    }

    // Sort deterministically by path to ensure output parity (e.g. for TF-IDF rendering order)
    all_features.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(all_features)
}

pub fn normalize_scope_filter(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_matches('/').trim_matches('\\');
    if trimmed.is_empty() || trimmed == "." {
        return None;
    }

    let normalized = normalize_path(Path::new(trimmed));
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub fn normalize_extension_filter(raw: &str) -> Option<String> {
    let normalized = raw.trim().trim_start_matches('.').to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub fn normalized_extension(path: &Path) -> String {
    path.extension()
        .map(|ext| ext.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default()
}

pub fn normalized_scope_terms(path: &Path) -> Vec<String> {
    let normalized = normalize_path(path);
    if normalized.is_empty() {
        return Vec::new();
    }

    let parts: Vec<&str> = normalized.split('/').collect();
    if parts.len() == 1 {
        return vec![normalized];
    }

    let mut terms = Vec::with_capacity(parts.len());
    for depth in 1..parts.len() {
        terms.push(parts[..depth].join("/"));
    }
    terms.push(normalized);
    terms
}

pub fn searchable_path_text(path: &Path) -> String {
    let normalized = normalize_path(path);
    if normalized.is_empty() {
        return String::new();
    }

    let spaced = normalized
        .chars()
        .map(|ch| match ch {
            '/' | '.' | '_' | '-' => ' ',
            _ => ch,
        })
        .collect::<String>();

    format!("{normalized} {spaced}")
}

fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("id", STRING | STORED);
    schema_builder.add_text_field("path", STRING | STORED);
    schema_builder.add_text_field("path_text", TEXT | STORED);
    schema_builder.add_text_field("scope_terms", STRING);
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("snippet", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT | STORED);
    schema_builder.add_text_field("file_type", STRING | STORED);
    schema_builder.add_text_field("extension", STRING | STORED);
    schema_builder.add_bytes_field("features", STORED);
    schema_builder.build()
}

fn has_compatible_schema(schema: &Schema) -> bool {
    REQUIRED_FIELDS
        .iter()
        .all(|field_name| schema.get_field(field_name).is_ok())
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => {
                let normalized = part.to_string_lossy().trim().to_ascii_lowercase();
                if normalized.is_empty() {
                    None
                } else {
                    Some(normalized)
                }
            }
            Component::CurDir => None,
            Component::ParentDir => Some("..".to_string()),
            Component::RootDir | Component::Prefix(_) => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::{normalize_extension_filter, normalize_scope_filter, normalized_scope_terms};
    use std::path::Path;

    #[test]
    fn normalizes_scope_filters_and_terms() {
        assert_eq!(
            normalize_scope_filter("./Alpha/Beta/"),
            Some("alpha/beta".to_string())
        );
        assert_eq!(
            normalized_scope_terms(Path::new("Alpha/Beta/File.md")),
            vec![
                "alpha".to_string(),
                "alpha/beta".to_string(),
                "alpha/beta/file.md".to_string()
            ]
        );
    }

    #[test]
    fn normalizes_extension_filters() {
        assert_eq!(normalize_extension_filter(".MD"), Some("md".to_string()));
        assert_eq!(normalize_extension_filter("   "), None);
    }
}
