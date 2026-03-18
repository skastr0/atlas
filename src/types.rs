//! Core data types for the indexer

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    Rust,
    JavaScript,
    Jsx,
    TypeScript,
    Tsx,
    Unknown,
}

impl FileType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" => Self::Markdown,
            "txt" | "text" | "json" | "yml" | "yaml" | "toml" | "sh" | "sql" => Self::PlainText,
            "pdf" => Self::Pdf,
            "rst" => Self::Rst,
            "org" => Self::Org,
            "rs" => Self::Rust,
            "js" | "mjs" | "cjs" => Self::JavaScript,
            "jsx" => Self::Jsx,
            "ts" => Self::TypeScript,
            "tsx" => Self::Tsx,
            _ => Self::Unknown,
        }
    }

    /// Check if this file type is source code
    pub fn is_code(&self) -> bool {
        matches!(
            self,
            Self::Rust | Self::JavaScript | Self::Jsx | Self::TypeScript | Self::Tsx
        )
    }

    pub fn search_term(&self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::PlainText => "plaintext",
            Self::Pdf => "pdf",
            Self::Rst => "rst",
            Self::Org => "org",
            Self::Rust => "rust",
            Self::JavaScript => "javascript",
            Self::Jsx => "jsx",
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
            Self::Unknown => "unknown",
        }
    }

    pub fn normalize_search_term(term: &str) -> String {
        term.trim()
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::FileType;

    #[test]
    fn maps_javascript_and_common_text_extensions() {
        assert_eq!(FileType::from_extension("js"), FileType::JavaScript);
        assert_eq!(FileType::from_extension("mjs"), FileType::JavaScript);
        assert_eq!(FileType::from_extension("cjs"), FileType::JavaScript);
        assert_eq!(FileType::from_extension("jsx"), FileType::Jsx);
        assert_eq!(FileType::from_extension("json"), FileType::PlainText);
        assert_eq!(FileType::from_extension("yaml"), FileType::PlainText);
        assert_eq!(FileType::from_extension("toml"), FileType::PlainText);
        assert_eq!(FileType::from_extension("sh"), FileType::PlainText);
        assert_eq!(FileType::from_extension("sql"), FileType::PlainText);
    }

    #[test]
    fn javascript_family_is_treated_as_code() {
        assert!(FileType::JavaScript.is_code());
        assert!(FileType::Jsx.is_code());
        assert!(!FileType::PlainText.is_code());
    }

    #[test]
    fn normalizes_search_terms_for_filters() {
        assert_eq!(FileType::normalize_search_term("plain-text"), "plaintext");
        assert_eq!(FileType::normalize_search_term("Type_Script"), "typescript");
        assert_eq!(FileType::Markdown.search_term(), "markdown");
    }
}

pub const SEARCH_RESULTS_CONTRACT_VERSION: u32 = 2;
pub const SCAN_RESULTS_CONTRACT_VERSION: u32 = 1;
pub const DOCTOR_RESULTS_CONTRACT_VERSION: u32 = 1;
pub const LAST_BUILD_MANIFEST_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanDeltaGroup {
    pub count: usize,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanDeltaGroups {
    pub new_files: ScanDeltaGroup,
    pub modified_files: ScanDeltaGroup,
    pub deleted_files: ScanDeltaGroup,
    pub unchanged_files: ScanDeltaGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanDeltaSummary {
    pub changed_files: usize,
    pub new_files: usize,
    pub modified_files: usize,
    pub deleted_files: usize,
    pub unchanged_files: usize,
    pub requires_build: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanDeltaReport {
    pub version: u32,
    pub read_only: bool,
    pub indexed_candidates: usize,
    pub summary: ScanDeltaSummary,
    pub groups: ScanDeltaGroups,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuildFileIssueReason {
    FileTooLarge,
    MetadataUnreadable,
    ExtractionFailed,
    PdftotextUnavailable,
}

impl BuildFileIssueReason {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FileTooLarge => "file_too_large",
            Self::MetadataUnreadable => "metadata_unreadable",
            Self::ExtractionFailed => "extraction_failed",
            Self::PdftotextUnavailable => "pdftotext_unavailable",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildFileIssue {
    pub path: String,
    pub reason: BuildFileIssueReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LastBuildManifest {
    pub version: u32,
    pub index_version: String,
    pub indexed_candidates: usize,
    pub indexed_documents: usize,
    pub processed_files: usize,
    pub full_reindex: bool,
    pub skipped: Vec<BuildFileIssue>,
    pub failed: Vec<BuildFileIssue>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DoctorState {
    Clean,
    Stale,
    Broken,
}

impl DoctorState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Stale => "stale",
            Self::Broken => "broken",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DoctorSeverity {
    Error,
    Warning,
    Info,
}

impl DoctorSeverity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorCheck {
    pub id: String,
    pub label: String,
    pub severity: DoctorSeverity,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorSeverityCounts {
    pub error: usize,
    pub warning: usize,
    pub info: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorSummary {
    pub indexed_candidates: usize,
    pub index_documents: usize,
    pub requires_build: bool,
    pub changed_files: usize,
    pub skipped_files: usize,
    pub failed_files: usize,
    pub severity_counts: DoctorSeverityCounts,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorCheckGroups {
    pub error: Vec<DoctorCheck>,
    pub warning: Vec<DoctorCheck>,
    pub info: Vec<DoctorCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub version: u32,
    pub state: DoctorState,
    pub summary: DoctorSummary,
    pub checks: DoctorCheckGroups,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchFilters {
    pub paths: Vec<String>,
    pub types: Vec<String>,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchQueryMetadata {
    pub text: String,
    pub limit: usize,
    pub filters: SearchFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResultItem {
    pub score: f32,
    pub path: String,
    pub file_type: FileType,
    pub extension: String,
    pub title: String,
    pub snippet: String,
    pub matched_fields: Vec<String>,
    pub highlight: SearchHighlight,
    pub reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResultsEnvelope {
    pub version: u32,
    pub index_version: String,
    pub query: SearchQueryMetadata,
    pub result_count: usize,
    pub results: Vec<SearchResultItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchHighlight {
    pub field: String,
    pub text: String,
    pub html: String,
    pub ranges: Vec<SearchHighlightRange>,
    #[serde(default)]
    pub fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchHighlightRange {
    pub start: usize,
    pub end: usize,
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
    pub terms: BTreeMap<String, TermStats>,
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
