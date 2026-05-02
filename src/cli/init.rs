//! `cmap init` command - Initialize .cmap directory

use crate::cache::tantivy_backend;
use crate::config::Config;
use crate::LogLevel;
use anyhow::{Context, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

const CMAP_DIR: &str = ".cmap";
const GITIGNORE_CMAP_ENTRY: &str = ".cmap/";

pub fn run(root: &Path, log_level: LogLevel) -> Result<()> {
    let cmap_path = root.join(CMAP_DIR);
    let already_initialized = cmap_path.exists();

    if already_initialized {
        if log_level != LogLevel::Quiet {
            println!("✓ .cmap already exists at {}", cmap_path.display());
        }
    } else {
        // Create directory structure
        fs::create_dir_all(cmap_path.join("cache/text"))
            .context("Failed to create cache/text directory")?;
        fs::create_dir_all(tantivy_backend::index_dir(&cmap_path))
            .context("Failed to create current tantivy index directory")?;
        fs::create_dir_all(cmap_path.join("global"))
            .context("Failed to create global directory")?;
        fs::create_dir_all(cmap_path.join("views/folders"))
            .context("Failed to create views/folders directory")?;

        // Write default config
        let config = Config::default();
        let config_path = cmap_path.join("config.toml");
        let config_toml = toml_string(&config);
        fs::write(&config_path, config_toml).context("Failed to write config.toml")?;
    }

    let gitignore_updated = ensure_gitignore_ignores_cmap(root)?;

    if log_level != LogLevel::Quiet {
        if !already_initialized {
            println!("✓ Initialized .cmap at {}", cmap_path.display());
            println!("  Edit .cmap/config.toml to customize settings");
        }
        if gitignore_updated {
            println!("✓ Added .cmap/ to {}", root.join(".gitignore").display());
        }
    }

    Ok(())
}

fn ensure_gitignore_ignores_cmap(root: &Path) -> Result<bool> {
    let gitignore_path = root.join(".gitignore");
    let existing = match fs::read_to_string(&gitignore_path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Failed to read {}", gitignore_path.display()))
        }
    };

    if gitignore_has_cmap_entry(&existing) {
        return Ok(false);
    }

    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(GITIGNORE_CMAP_ENTRY);
    updated.push('\n');

    fs::write(&gitignore_path, updated)
        .with_context(|| format!("Failed to write {}", gitignore_path.display()))?;

    Ok(true)
}

fn gitignore_has_cmap_entry(content: &str) -> bool {
    content
        .lines()
        .any(|line| matches!(line.trim(), ".cmap" | ".cmap/" | "/.cmap" | "/.cmap/"))
}

fn toml_string(config: &Config) -> String {
    // Manual TOML generation to keep comments and ordering stable
    let mut s = String::new();

    s.push_str("[scan]\n");
    s.push_str("# Patterns to ignore (in addition to .gitignore)\n");
    s.push_str("ignore = [\n");
    for pattern in &config.scan.ignore {
        s.push_str(&format!("    \"{}\",\n", pattern));
    }
    s.push_str("]\n\n");

    s.push_str("# File extensions to index\n");
    s.push_str("include_extensions = [\n");
    for ext in &config.scan.include_extensions {
        s.push_str(&format!("    \"{}\",\n", ext));
    }
    s.push_str("]\n\n");

    s.push_str("[extract]\n");
    s.push_str(&format!(
        "# Max file size to process (bytes)\nmax_file_size = {}\n\n",
        config.extract.max_file_size
    ));
    s.push_str(&format!(
        "# Snippet length (chars)\nsnippet_length = {}\n\n",
        config.extract.snippet_length
    ));
    s.push_str("# Path to pdftotext binary (auto-detected if not set)\n");
    s.push_str("# pdftotext_path = \"/usr/bin/pdftotext\"\n\n");

    s.push_str("[analyze]\n");
    s.push_str(&format!(
        "# Number of top terms per file\ntop_terms = {}\n\n",
        config.analyze.top_terms
    ));
    s.push_str(&format!(
        "# Number of top phrases per file\ntop_phrases = {}\n\n",
        config.analyze.top_phrases
    ));
    s.push_str(&format!(
        "# Minimum term length\nmin_term_length = {}\n\n",
        config.analyze.min_term_length
    ));
    s.push_str(&format!(
        "# Maximum term length\nmax_term_length = {}\n\n",
        config.analyze.max_term_length
    ));
    s.push_str(&format!(
        "# Maximum digit ratio in a term (0.0-1.0)\nmax_digit_ratio = {}\n\n",
        config.analyze.max_digit_ratio
    ));
    s.push_str(&format!(
        "# Minimum document frequency for a term\nmin_df = {}\n\n",
        config.analyze.min_df
    ));
    s.push_str(&format!(
        "# Maximum document frequency ratio (0.0-1.0)\nmax_df_ratio = {}\n\n",
        config.analyze.max_df_ratio
    ));

    s.push_str("[render]\n");
    s.push_str(&format!(
        "# Folder depth in ROOT_ATLAS.md\natlas_folder_depth = {}\n\n",
        config.render.atlas_folder_depth
    ));
    s.push_str(&format!(
        "# Max files to list per folder in atlas\natlas_max_files_per_folder = {}\n",
        config.render.atlas_max_files_per_folder
    ));

    s
}

#[cfg(test)]
mod tests {
    use super::{gitignore_has_cmap_entry, run, toml_string};
    use crate::config::{Config, DEFAULT_INCLUDE_EXTENSIONS};
    use crate::LogLevel;
    use std::fs;

    fn run_init_with_gitignore(content: Option<&str>) -> String {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let root = temp.path();
        let gitignore_path = root.join(".gitignore");

        if let Some(content) = content {
            fs::write(&gitignore_path, content).expect("gitignore should be seeded");
        }

        run(root, LogLevel::Quiet).expect("init should succeed");
        run(root, LogLevel::Quiet).expect("second init should succeed");

        fs::read_to_string(&gitignore_path).expect("gitignore should be readable")
    }

    #[test]
    fn renders_default_extensions_into_generated_config() {
        let rendered = toml_string(&Config::default());
        let parsed: Config = toml::from_str(&rendered).expect("generated config should parse");
        let expected: Vec<String> = DEFAULT_INCLUDE_EXTENSIONS
            .iter()
            .map(|ext| (*ext).to_string())
            .collect();

        assert_eq!(parsed.scan.include_extensions, expected);
    }

    #[test]
    fn initializes_cmap_and_adds_gitignore_entry_idempotently() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let root = temp.path();
        let gitignore_path = root.join(".gitignore");

        fs::write(&gitignore_path, "target\n").expect("gitignore should be seeded");

        run(root, LogLevel::Quiet).expect("init should succeed");
        run(root, LogLevel::Quiet).expect("second init should succeed");

        let gitignore = fs::read_to_string(&gitignore_path).expect("gitignore should be readable");
        assert!(root.join(".cmap/config.toml").is_file());
        assert_eq!(gitignore.matches(".cmap/").count(), 1);
        assert!(gitignore_has_cmap_entry(&gitignore));
    }

    #[test]
    fn creates_missing_gitignore_with_cmap_entry() {
        let gitignore = run_init_with_gitignore(None);

        assert_eq!(gitignore, ".cmap/\n");
    }

    #[test]
    fn adds_missing_gitignore_entry_when_cmap_already_exists() {
        let temp = tempfile::tempdir().expect("tempdir should be created");
        let root = temp.path();
        let gitignore_path = root.join(".gitignore");

        fs::create_dir_all(root.join(".cmap")).expect(".cmap should be seeded");

        run(root, LogLevel::Quiet).expect("init should succeed");
        run(root, LogLevel::Quiet).expect("second init should succeed");

        let gitignore = fs::read_to_string(&gitignore_path).expect("gitignore should be readable");
        assert_eq!(gitignore, ".cmap/\n");
    }

    #[test]
    fn recognizes_existing_cmap_gitignore_variants() {
        for variant in [".cmap", ".cmap/", "/.cmap", "/.cmap/"] {
            let gitignore = run_init_with_gitignore(Some(&format!("target\n{variant}\n")));

            assert_eq!(gitignore, format!("target\n{variant}\n"));
            assert!(gitignore_has_cmap_entry(&gitignore));
        }

        assert!(!gitignore_has_cmap_entry("# .cmap/\n"));
    }
}
