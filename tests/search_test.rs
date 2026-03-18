use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[test]
fn search_supports_path_type_and_ext_filters_with_stable_json_envelope() {
    let fixture_root = TempDir::new().expect("create temp corpus root");
    write_test_files(fixture_root.path()).expect("write test files");

    run_cmap(fixture_root.path(), &["init"]);
    run_cmap(fixture_root.path(), &["build"]);

    let output = run_cmap_output(
        fixture_root.path(),
        &[
            "search",
            "programming",
            "--path",
            "alpha",
            "--path",
            "beta",
            "--type",
            "markdown",
            "--type",
            "rust",
            "--ext",
            "md",
            "--ext",
            ".rs",
            "--json",
            "--limit",
            "5",
        ],
    );
    assert!(output.status.success(), "search command should succeed");

    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("search output should be valid JSON");

    assert_eq!(
        parsed.get("version").and_then(|value| value.as_u64()),
        Some(2)
    );
    assert_eq!(
        parsed.get("index_version").and_then(|value| value.as_str()),
        Some("tantivy-v2")
    );
    assert_eq!(
        parsed
            .pointer("/query/text")
            .and_then(|value| value.as_str()),
        Some("programming")
    );
    assert_eq!(
        parsed
            .pointer("/query/limit")
            .and_then(|value| value.as_u64()),
        Some(5)
    );
    assert_eq!(
        parsed
            .pointer("/query/filters/paths")
            .and_then(|value| value.as_array())
            .unwrap(),
        &vec![serde_json::json!("alpha"), serde_json::json!("beta")]
    );
    assert_eq!(
        parsed
            .pointer("/query/filters/types")
            .and_then(|value| value.as_array())
            .unwrap(),
        &vec![serde_json::json!("markdown"), serde_json::json!("rust")]
    );
    assert_eq!(
        parsed
            .pointer("/query/filters/extensions")
            .and_then(|value| value.as_array())
            .unwrap(),
        &vec![serde_json::json!("md"), serde_json::json!("rs")]
    );

    let results = parsed
        .get("results")
        .and_then(|value| value.as_array())
        .expect("results should be an array");
    let paths: Vec<&str> = results
        .iter()
        .map(|result| result.get("path").unwrap().as_str().unwrap())
        .collect();

    assert_eq!(paths.len(), 2);
    assert!(paths.contains(&"alpha/file1.md"));
    assert!(paths.contains(&"beta/file4.rs"));

    let alpha_result = results
        .iter()
        .find(|result| result.get("path").unwrap().as_str() == Some("alpha/file1.md"))
        .expect("alpha markdown result should exist");
    let beta_result = results
        .iter()
        .find(|result| result.get("path").unwrap().as_str() == Some("beta/file4.rs"))
        .expect("beta rust result should exist");

    assert_eq!(
        alpha_result.get("file_type").unwrap().as_str(),
        Some("markdown")
    );
    assert_eq!(alpha_result.get("extension").unwrap().as_str(), Some("md"));
    assert_eq!(beta_result.get("file_type").unwrap().as_str(), Some("rust"));
    assert_eq!(beta_result.get("extension").unwrap().as_str(), Some("rs"));
}

#[test]
fn search_applies_scope_filters_before_ranking() {
    let fixture_root = TempDir::new().expect("create temp corpus root");
    write_test_files(fixture_root.path()).expect("write test files");

    run_cmap(fixture_root.path(), &["init"]);
    run_cmap(fixture_root.path(), &["build"]);

    let output = run_cmap_output(
        fixture_root.path(),
        &[
            "search",
            "rust systems",
            "--path",
            "alpha",
            "--limit",
            "1",
            "--json",
        ],
    );
    assert!(output.status.success(), "search command should succeed");

    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("search output should be valid JSON");
    let results = parsed
        .get("results")
        .and_then(|value| value.as_array())
        .expect("results should be an array");

    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].get("path").unwrap().as_str(),
        Some("alpha/file1.md")
    );
}

#[test]
fn search_rebuilds_when_current_index_version_is_missing() {
    let fixture_root = TempDir::new().expect("create temp corpus root");
    write_test_files(fixture_root.path()).expect("write test files");

    run_cmap(fixture_root.path(), &["init"]);
    run_cmap(fixture_root.path(), &["build"]);

    let current_index = fixture_root.path().join(".cmap/index/tantivy-v2");
    let old_index = fixture_root.path().join(".cmap/index/tantivy-v1");
    fs::rename(&current_index, &old_index).expect("rename current index to old version");

    let failed_search = run_cmap_output(fixture_root.path(), &["search", "rust"]);
    assert!(
        !failed_search.status.success(),
        "search should fail without v2 index"
    );
    let stderr = String::from_utf8_lossy(&failed_search.stderr);
    assert!(stderr.contains("tantivy-v2"));
    assert!(stderr.contains("cmap build"));

    run_cmap(fixture_root.path(), &["build"]);
    assert!(
        current_index.exists(),
        "build should recreate the current index version"
    );

    let rebuilt_search = run_cmap_output(
        fixture_root.path(),
        &["search", "programming", "--json", "--limit", "10"],
    );
    assert!(
        rebuilt_search.status.success(),
        "search should succeed after rebuild"
    );

    let parsed: serde_json::Value =
        serde_json::from_slice(&rebuilt_search.stdout).expect("search output should be valid JSON");
    let paths: Vec<&str> = parsed["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|result| result["path"].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"alpha/file1.md"));
    assert!(paths.contains(&"beta/file4.rs"));
}

#[test]
fn search_breaks_score_ties_by_path() {
    let fixture_root = TempDir::new().expect("create temp corpus root");
    write_test_files(fixture_root.path()).expect("write test files");

    run_cmap(fixture_root.path(), &["init"]);
    run_cmap(fixture_root.path(), &["build"]);

    let output = run_cmap_output(
        fixture_root.path(),
        &["search", "sharedterm", "--json", "--limit", "10"],
    );
    assert!(output.status.success(), "search command should succeed");

    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("search output should be valid JSON");
    let paths: Vec<&str> = parsed["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|result| result["path"].as_str().unwrap())
        .collect();

    assert_eq!(paths, vec!["alpha/tie-a.md", "beta/tie-b.md"]);
}

fn write_test_files(root: &Path) -> std::io::Result<()> {
    write_file(
        root.join("alpha/file1.md"),
        "# Rust Programming\n\nRust programming for systems teams.\n",
    )?;
    write_file(
        root.join("alpha/file2.txt"),
        "rust programming notes in plain text only\n",
    )?;
    write_file(
        root.join("beta/file3.md"),
        "# Rust Systems Rust Systems Rust Systems\n\nrust systems rust systems rust systems rust systems\n",
    )?;
    write_file(
        root.join("beta/file4.rs"),
        "fn programming_rust_guide() {\n    let _topic = \"programming rust guide\";\n}\n",
    )?;
    write_file(
        root.join("alpha/tie-a.md"),
        "# Shared Match\n\nsharedterm sharedterm\n",
    )?;
    write_file(
        root.join("beta/tie-b.md"),
        "# Shared Match\n\nsharedterm sharedterm\n",
    )?;

    Ok(())
}

fn write_file(path: PathBuf, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

fn run_cmap(root: &Path, args: &[&str]) {
    let output = run_cmap_output(root, args);
    assert!(
        output.status.success(),
        "cmap command failed: {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_cmap_output(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cmap"))
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .expect("execute cmap command")
}
