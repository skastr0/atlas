use crate::types::FileType;
use tree_sitter::{Language as TsLanguage, Parser, Tree};

/// Supported tree-sitter languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Markdown,
    Rust,
    TypeScript,
    Tsx,
}

impl Language {
    pub fn from_file_type(file_type: FileType) -> Option<Self> {
        match file_type {
            FileType::Markdown => Some(Self::Markdown),
            FileType::Rust => Some(Self::Rust),
            FileType::TypeScript => Some(Self::TypeScript),
            FileType::Tsx => Some(Self::Tsx),
            _ => None,
        }
    }

    pub fn grammar(&self) -> TsLanguage {
        match self {
            Self::Markdown => tree_sitter_md::LANGUAGE.into(),
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        }
    }
}

/// Parse source code into a tree-sitter AST.
pub fn parse(source: &str, language: Language) -> Option<Tree> {
    let mut parser = Parser::new();
    let grammar = language.grammar();

    if parser.set_language(&grammar).is_err() {
        return None;
    }

    parser.parse(source, None)
}

#[cfg(test)]
mod tests {
    use super::{parse, Language};

    #[test]
    fn parses_simple_rust_snippet() {
        let tree = parse("fn main() {}", Language::Rust);
        assert!(tree.is_some());
    }
}
