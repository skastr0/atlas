use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

#[test]
fn doctor_reports_clean_fixture_in_json() {
    let fixture_root = TempDir::new().expect("create temp corpus root");
    write_baseline_files(fixture_root.path()).expect("write baseline files");

    run_atlas(fixture_root.path(), &["init"]);
    run_atlas(fixture_root.path(), &["build"]);

    let output = run_atlas_output(fixture_root.path(), &["doctor", "--json"]);
    assert!(output.status.success(), "doctor command should succeed");
    let parsed = parse_json(&output);

    assert_eq!(parsed["version"].as_u64(), Some(1));
    assert_eq!(parsed["state"].as_str(), Some("clean"));
    assert_eq!(parsed["summary"]["indexed_candidates"].as_u64(), Some(3));
    assert_eq!(parsed["summary"]["index_documents"].as_u64(), Some(3));
    assert_eq!(parsed["summary"]["changed_files"].as_u64(), Some(0));
    assert_eq!(parsed["summary"]["skipped_files"].as_u64(), Some(0));
    assert_eq!(parsed["summary"]["failed_files"].as_u64(), Some(0));
    assert_eq!(parsed["summary"]["requires_build"].as_bool(), Some(false));
    assert_eq!(parsed["checks"]["error"], serde_json::json!([]));
    assert_eq!(parsed["checks"]["warning"], serde_json::json!([]));

    let info_ids = collect_check_ids(&parsed["checks"]["info"]);
    for expected in [
        "initialization",
        "config",
        "fingerprints",
        "last_build_manifest",
        "index",
        "generated_artifacts",
        "corpus_delta",
        "index_drift",
        "last_build_skips",
        "last_build_failures",
    ] {
        assert!(
            info_ids.contains(&expected.to_string()),
            "expected info check `{expected}` in {:?}",
            info_ids
        );
    }
    assert!(
        !info_ids.contains(&"pdf_dependency".to_string()),
        "clean fixture should not require pdf dependency checks"
    );

    let manifest = read_json_file(fixture_root.path().join(".atlas/last-build.json"));
    assert_eq!(manifest["version"].as_u64(), Some(1));
    assert_eq!(manifest["index_version"].as_str(), Some("tantivy-v2"));
    assert_eq!(manifest["indexed_candidates"].as_u64(), Some(3));
    assert_eq!(manifest["indexed_documents"].as_u64(), Some(3));
    assert_eq!(manifest["processed_files"].as_u64(), Some(3));
    assert_eq!(manifest["skipped"], serde_json::json!([]));
    assert_eq!(manifest["failed"], serde_json::json!([]));
}

#[test]
fn doctor_reports_stale_fixture_and_groups_human_output_by_severity() {
    let fixture_root = TempDir::new().expect("create temp corpus root");
    write_baseline_files(fixture_root.path()).expect("write baseline files");
    write_file(
        fixture_root.path().join("docs/oversized.md"),
        "# Oversized\n\nthis file is intentionally larger than the configured extraction limit\n",
    )
    .expect("write oversized file");

    run_atlas(fixture_root.path(), &["init"]);
    write_config(fixture_root.path(), "[extract]\nmax_file_size = 32\n")
        .expect("write reduced max file size config");
    run_atlas(fixture_root.path(), &["build"]);

    write_file(
        fixture_root.path().join("docs/new.md"),
        "# New\n\nthis file was added after the last build\n",
    )
    .expect("write new file after build");

    let json_output = run_atlas_output(fixture_root.path(), &["doctor", "--json"]);
    assert!(
        json_output.status.success(),
        "doctor command should succeed"
    );
    let parsed = parse_json(&json_output);

    assert_eq!(parsed["state"].as_str(), Some("stale"));
    assert_eq!(parsed["summary"]["changed_files"].as_u64(), Some(1));
    assert_eq!(parsed["summary"]["skipped_files"].as_u64(), Some(1));
    assert_eq!(parsed["summary"]["failed_files"].as_u64(), Some(0));
    assert_eq!(parsed["checks"]["error"], serde_json::json!([]));

    let warning_ids = collect_check_ids(&parsed["checks"]["warning"]);
    for expected in ["corpus_delta", "index_drift", "last_build_skips"] {
        assert!(
            warning_ids.contains(&expected.to_string()),
            "expected warning check `{expected}` in {:?}",
            warning_ids
        );
    }

    let warning_blob = serde_json::to_string(&parsed["checks"]["warning"])
        .expect("warning checks should serialize");
    assert!(warning_blob.contains("docs/new.md"));
    assert!(warning_blob.contains("docs/oversized.md"));
    assert!(warning_blob.contains("file_too_large"));

    let human_output = run_atlas_output(fixture_root.path(), &["doctor"]);
    assert!(
        human_output.status.success(),
        "doctor command should succeed"
    );
    let stdout = String::from_utf8_lossy(&human_output.stdout);
    assert!(stdout.contains("State: stale"));
    assert!(stdout.contains("Warnings ("));
    assert!(stdout.contains("Info ("));
    assert!(!stdout.contains("Errors ("));
    assert!(stdout.contains("- Corpus delta:"));
    assert!(stdout.contains("- Last build skips:"));
    assert!(stdout.contains("docs/oversized.md (file_too_large"));
}

#[test]
fn doctor_reports_broken_fixture_when_generated_artifacts_are_missing() {
    let fixture_root = TempDir::new().expect("create temp corpus root");
    write_baseline_files(fixture_root.path()).expect("write baseline files");

    run_atlas(fixture_root.path(), &["init"]);
    run_atlas(fixture_root.path(), &["build"]);
    fs::remove_file(fixture_root.path().join(".atlas/views/TERMS.md"))
        .expect("remove generated artifact");

    let output = run_atlas_output(fixture_root.path(), &["doctor", "--json"]);
    assert!(output.status.success(), "doctor command should succeed");
    let parsed = parse_json(&output);

    assert_eq!(parsed["state"].as_str(), Some("broken"));
    assert_eq!(parsed["summary"]["requires_build"].as_bool(), Some(true));

    let error_ids = collect_check_ids(&parsed["checks"]["error"]);
    assert!(error_ids.contains(&"generated_artifacts".to_string()));
    let error_blob =
        serde_json::to_string(&parsed["checks"]["error"]).expect("error checks should serialize");
    assert!(error_blob.contains("views/TERMS.md"));
}

#[test]
fn doctor_reports_missing_pdf_dependency_and_failed_manifest_reason() {
    let fixture_root = TempDir::new().expect("create temp corpus root");
    write_file(
        fixture_root.path().join("docs/alpha.md"),
        "# Alpha\n\nmarkdown still builds\n",
    )
    .expect("write markdown file");
    write_file(
        fixture_root.path().join("docs/guide.pdf"),
        "%PDF-1.4\n1 0 obj\n<<>>\nendobj\n",
    )
    .expect("write fake pdf");

    run_atlas(fixture_root.path(), &["init"]);
    write_config(
        fixture_root.path(),
        "[extract]\npdftotext_path = \"/definitely/missing/pdftotext\"\n",
    )
    .expect("write invalid pdftotext path config");
    run_atlas(fixture_root.path(), &["build"]);

    let output = run_atlas_output(fixture_root.path(), &["doctor", "--json"]);
    assert!(output.status.success(), "doctor command should succeed");
    let parsed = parse_json(&output);

    assert_eq!(parsed["state"].as_str(), Some("broken"));
    assert_eq!(parsed["summary"]["failed_files"].as_u64(), Some(1));

    let error_ids = collect_check_ids(&parsed["checks"]["error"]);
    assert!(error_ids.contains(&"last_build_failures".to_string()));
    assert!(error_ids.contains(&"pdf_dependency".to_string()));

    let error_blob =
        serde_json::to_string(&parsed["checks"]["error"]).expect("error checks should serialize");
    assert!(error_blob.contains("docs/guide.pdf"));
    assert!(error_blob.contains("pdftotext_unavailable"));
    assert!(error_blob.contains("configured pdftotext_path"));

    let manifest = read_json_file(fixture_root.path().join(".atlas/last-build.json"));
    assert_eq!(
        manifest["failed"][0]["path"].as_str(),
        Some("docs/guide.pdf")
    );
    assert_eq!(
        manifest["failed"][0]["reason"].as_str(),
        Some("pdftotext_unavailable")
    );
}

fn write_baseline_files(root: &Path) -> std::io::Result<()> {
    write_file(root.join("docs/alpha.md"), "# Alpha\n\nalpha content\n")?;
    write_file(root.join("docs/beta.md"), "# Beta\n\nbeta content\n")?;
    write_file(root.join("src/lib.rs"), "pub fn helper() {}\n")?;
    Ok(())
}

fn write_config(root: &Path, content: &str) -> std::io::Result<()> {
    fs::write(root.join(".atlas/config.toml"), content)
}

fn write_file(path: PathBuf, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

fn parse_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("doctor output should be valid JSON")
}

fn read_json_file(path: PathBuf) -> serde_json::Value {
    let bytes = fs::read(path).expect("json file should be readable");
    serde_json::from_slice(&bytes).expect("json file should be valid")
}

fn collect_check_ids(group: &serde_json::Value) -> Vec<String> {
    group
        .as_array()
        .expect("check group should be an array")
        .iter()
        .map(|check| {
            check["id"]
                .as_str()
                .expect("check should include an id")
                .to_string()
        })
        .collect()
}

fn run_atlas(root: &Path, args: &[&str]) {
    let output = run_atlas_output(root, args);
    assert!(
        output.status.success(),
        "atlas command failed: {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_atlas_output(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_atlas"))
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .expect("execute atlas command")
}
