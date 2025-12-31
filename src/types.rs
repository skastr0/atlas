//! Core data types for the indexer

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// File fingerprint for change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fingerprint {
    /// Relative path from knowledge base root
    pub path: PathBuf,
    /// Last modification time (Unix timestamp)
    pub mtime: u64,
    /// File size in bytes
    pub size: u64,
    /// Content hash (Blake3, computed on demand)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

/// Type of file being indexed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Markdown,
    PlainText,
    Pdf,
    Rst,
    Org,
    Unknown,
}

impl FileType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" => Self::Markdown,
            "txt" | "text" => Self::PlainText,
            "pdf" => Self::Pdf,
            "rst" => Self::Rst,
            "org" => Self::Org,
            _ => Self::Unknown,
        }
    }
}

/// Per-file extracted features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFeatures {
    /// Unique identifier (content hash)
    pub id: String,
    /// Relative path from knowledge base root
    pub path: PathBuf,
    /// Detected file type
    pub file_type: FileType,
    /// Extracted or derived title
    pub title: String,
    /// First paragraph or ~400 chars
    pub snippet: String,
    /// Total word count
    pub word_count: usize,
    /// Total character count
    pub char_count: usize,
    /// Unique term count before truncation
    #[serde(default)]
    pub unique_term_count: usize,
    /// Top terms by TF-IDF (after global pass)
    pub top_terms: Vec<TermScore>,
    /// Top bigrams/trigrams
    pub top_phrases: Vec<PhraseScore>,
    /// Top RAKE phrases
    #[serde(default)]
    pub rake_phrases: Vec<PhraseScore>,
    /// Top YAKE keywords (lower score = more important)
    #[serde(default)]
    pub yake_keywords: Vec<KeywordScore>,
    /// Extracted links (internal/external)
    pub links_out: Vec<Link>,
    /// Extracted headings (if available)
    pub headings: Vec<String>,
    /// Did extraction succeed?
    pub extraction_ok: bool,
    /// Unix timestamp of extraction
    pub extracted_at: u64,
}

/// Term with TF-IDF score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermScore {
    /// The term (lowercase, normalized)
    pub term: String,
    /// Term frequency in this document
    pub tf: f32,
    /// TF-IDF score (computed after global pass)
    #[serde(default)]
    pub tfidf: f32,
}

/// Phrase with score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhraseScore {
    /// The phrase (normalized)
    pub phrase: String,
    /// Phrase score (RAKE or frequency-based)
    #[serde(default, alias = "count")]
    pub score: f32,
}

/// Keyword with YAKE score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordScore {
    /// The keyword or phrase
    pub keyword: String,
    /// YAKE score (lower is better)
    pub score: f32,
}

/// Link type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    /// Internal link (relative path or wiki-link)
    Internal,
    /// External URL
    External,
    /// Citation (DOI, arXiv, ISBN, etc.)
    Citation,
}

/// Extracted link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// Link target (path or URL)
    pub target: String,
    /// Type of link
    pub link_type: LinkType,
}

/// Global term document frequency index
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalTermIndex {
    /// Total number of documents indexed
    pub total_docs: usize,
    /// Term -> stats mapping
    pub terms: HashMap<String, TermStats>,
}

/// Statistics for a single term across the corpus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermStats {
    /// Document frequency (number of docs containing this term)
    pub df: usize,
    /// Top documents by TF-IDF for this term (file IDs)
    pub top_docs: Vec<String>,
}

/// Folder signature (aggregate stats for a folder)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderSignature {
    /// Relative path from knowledge base root
    pub path: PathBuf,
    /// Number of indexed files in this folder (recursive)
    pub file_count: usize,
    /// Top distinctive terms for this folder
    pub top_terms: Vec<String>,
    /// Top distinctive phrases for this folder
    pub top_phrases: Vec<String>,
}

/// Change status for a file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeStatus {
    /// File is new (not in previous fingerprints)
    New,
    /// File was modified (mtime or size changed)
    Modified,
    /// File was deleted (in previous fingerprints, not on disk)
    Deleted,
    /// File is unchanged
    Unchanged,
}

/// Result of scanning for changes
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    /// All current fingerprints
    pub fingerprints: Vec<Fingerprint>,
    /// Files that are new
    pub new_files: Vec<PathBuf>,
    /// Files that were modified
    pub modified_files: Vec<PathBuf>,
    /// Files that were deleted
    pub deleted_files: Vec<PathBuf>,
    /// Total files scanned
    pub total_files: usize,
}
