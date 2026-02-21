//! Tantivy backend for feature cache

use crate::types::FileFeatures;
use anyhow::{Context, Result};
use std::path::Path;
use tantivy::schema::{Schema, Value, STORED, STRING};
use tantivy::{doc, Index, IndexWriter, TantivyDocument, Term};

/// Initialize or open a Tantivy index at the given directory
pub fn init_index(index_dir: &Path) -> Result<Index> {
    if !index_dir.exists() {
        std::fs::create_dir_all(index_dir).context("Failed to create tantivy index directory")?;
    }

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("id", STRING | STORED);
    schema_builder.add_bytes_field("features", STORED);
    let schema = schema_builder.build();

    let index = if index_dir.join("meta.json").exists() {
        Index::open_in_dir(index_dir).context("Failed to open existing tantivy index")?
    } else {
        Index::create_in_dir(index_dir, schema.clone()).context("Failed to create tantivy index")?
    };

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
    features_list: &[FileFeatures],
) -> Result<()> {
    let schema = index.schema();
    let id_field = schema.get_field("id").unwrap();
    let features_field = schema.get_field("features").unwrap();

    for features in features_list {
        let serialized = serde_json::to_vec(features).context("Failed to serialize features")?;
        writer
            .add_document(doc!(
                id_field => features.id.clone(),
                features_field => serialized
            ))
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
