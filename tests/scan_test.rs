use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

#[test]
fn scan_reports_noop_delta_and_keeps_dry_run_compatibility() {
    let fixture_root = TempDir::new().expect("create temp corpus root");
    write_baseline_files(fixture_root.path()).expect("write baseline files");

    run_cmap(fixture_root.path(), &["init"]);
    run_cmap(fixture_root.path(), &["build"]);

    let json_output = run_cmap_output(fixture_root.path(), &["scan", "--json"]);
    assert!(json_output.status.success(), "scan command should succeed");
    let parsed = parse_json(&json_output);

    assert_eq!(parsed["version"].as_u64(), Some(1));
    assert_eq!(parsed["read_only"].as_bool(), Some(true));
    assert_eq!(parsed["indexed_candidates"].as_u64(), Some(3));
    assert_eq!(parsed["summary"]["changed_files"].as_u64(), Some(0));
    assert_eq!(parsed["summary"]["new_files"].as_u64(), Some(0));
    assert_eq!(parsed["summary"]["modified_files"].as_u64(), Some(0));
    assert_eq!(parsed["summary"]["deleted_files"].as_u64(), Some(0));
    assert_eq!(parsed["summary"]["unchanged_files"].as_u64(), Some(3));
    assert_eq!(parsed["summary"]["requires_build"].as_bool(), Some(false));
    assert_eq!(
        parsed["groups"]["unchanged_files"]["paths"],
        serde_json::json!(["docs/alpha.md", "docs/beta.md", "src/lib.rs"])
    );

    let dry_run_output = run_cmap_output(fixture_root.path(), &["scan", "--dry-run", "--json"]);
    assert!(
        dry_run_output.status.success(),
        "scan --dry-run should succeed"
    );
    assert_eq!(parsed, parse_json(&dry_run_output));
}

#[test]
fn scan_reports_new_modified_and_deleted_files_without_mutating_cmap_state() {
    let fixture_root = TempDir::new().expect("create temp corpus root");
    write_baseline_files(fixture_root.path()).expect("write baseline files");

    run_cmap(fixture_root.path(), &["init"]);
    run_cmap(fixture_root.path(), &["build"]);

    let before_scan = snapshot_cmap(fixture_root.path()).expect("snapshot cmap before scan");

    write_file(
        fixture_root.path().join("docs/beta.md"),
        "# Beta\n\nmodified content that changes size\n",
    )
    .expect("modify beta file");
    write_file(
        fixture_root.path().join("docs/new-file.md"),
        "# New\n\nthis file is new\n",
    )
    .expect("write new file");
    fs::remove_file(fixture_root.path().join("src/lib.rs")).expect("delete tracked file");

    let output = run_cmap_output(fixture_root.path(), &["scan", "--json"]);
    assert!(output.status.success(), "scan command should succeed");
    let parsed = parse_json(&output);

    assert_eq!(parsed["indexed_candidates"].as_u64(), Some(3));
    assert_eq!(parsed["summary"]["changed_files"].as_u64(), Some(3));
    assert_eq!(parsed["summary"]["new_files"].as_u64(), Some(1));
    assert_eq!(parsed["summary"]["modified_files"].as_u64(), Some(1));
    assert_eq!(parsed["summary"]["deleted_files"].as_u64(), Some(1));
    assert_eq!(parsed["summary"]["unchanged_files"].as_u64(), Some(1));
    assert_eq!(parsed["summary"]["requires_build"].as_bool(), Some(true));
    assert_eq!(
        parsed["groups"]["new_files"]["paths"],
        serde_json::json!(["docs/new-file.md"])
    );
    assert_eq!(
        parsed["groups"]["modified_files"]["paths"],
        serde_json::json!(["docs/beta.md"])
    );
    assert_eq!(
        parsed["groups"]["deleted_files"]["paths"],
        serde_json::json!(["src/lib.rs"])
    );
    assert_eq!(
        parsed["groups"]["unchanged_files"]["paths"],
        serde_json::json!(["docs/alpha.md"])
    );

    let after_scan = snapshot_cmap(fixture_root.path()).expect("snapshot cmap after scan");
    assert_eq!(before_scan, after_scan, "scan must be read-only");
}

#[test]
fn scan_human_output_bounds_representative_paths_and_highlights_build_impact() {
    let fixture_root = TempDir::new().expect("create temp corpus root");
    write_baseline_files(fixture_root.path()).expect("write baseline files");

    run_cmap(fixture_root.path(), &["init"]);
    run_cmap(fixture_root.path(), &["build"]);

    fs::remove_file(fixture_root.path().join("src/lib.rs")).expect("delete tracked file");
    for index in 0..7 {
        write_file(
            fixture_root.path().join(format!("docs/new-{index:02}.md")),
            &format!("# New {index}\n\nextra content {index}\n"),
        )
        .expect("write new file");
    }

    let output = run_cmap_output(fixture_root.path(), &["scan"]);
    assert!(output.status.success(), "scan command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Indexed candidates: 9"));
    assert!(
        stdout.contains("Build impact: 8 changed path(s) would be reprocessed by the next build")
    );
    assert!(stdout.contains("New: 7 | Modified: 0 | Deleted: 1 | Unchanged: 2"));
    assert!(stdout.contains("New files (7):"));
    assert!(stdout.contains("  - docs/new-00.md"));
    assert!(stdout.contains("  - docs/new-04.md"));
    assert!(!stdout.contains("  - docs/new-05.md"));
    assert!(stdout.contains("  ... 2 more"));
    assert!(stdout.contains("Deleted files (1):"));
    assert!(stdout.contains("  - src/lib.rs"));
}

#[test]
fn scan_fails_when_repo_is_not_initialized_or_config_is_unreadable() {
    let missing_root = TempDir::new().expect("create missing init root");
    let missing_output = run_cmap_output(missing_root.path(), &["scan"]);
    assert!(
        !missing_output.status.success(),
        "scan should fail without .cmap"
    );
    let missing_stderr = String::from_utf8_lossy(&missing_output.stderr);
    assert!(missing_stderr.contains("Not initialized"));
    assert!(missing_stderr.contains("cmap init"));

    let broken_root = TempDir::new().expect("create broken config root");
    write_baseline_files(broken_root.path()).expect("write baseline files");
    run_cmap(broken_root.path(), &["init"]);

    let config_path = broken_root.path().join(".cmap/config.toml");
    fs::remove_file(&config_path).expect("remove config file");
    fs::create_dir(&config_path).expect("replace config file with directory");

    let broken_output = run_cmap_output(broken_root.path(), &["scan"]);
    assert!(
        !broken_output.status.success(),
        "scan should fail when config cannot be read"
    );
    let broken_stderr = String::from_utf8_lossy(&broken_output.stderr);
    assert!(broken_stderr.contains("Failed to load scan config"));
    assert!(broken_stderr.contains("config.toml"));
}

fn write_baseline_files(root: &Path) -> std::io::Result<()> {
    write_file(root.join("docs/alpha.md"), "# Alpha\n\nalpha content\n")?;
    write_file(root.join("docs/beta.md"), "# Beta\n\nbeta content\n")?;
    write_file(root.join("src/lib.rs"), "pub fn helper() {}\n")?;
    Ok(())
}

fn write_file(path: PathBuf, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

fn snapshot_cmap(root: &Path) -> std::io::Result<BTreeMap<String, String>> {
    let cmap_root = root.join(".cmap");
    let mut snapshot = BTreeMap::new();
    collect_snapshot(&cmap_root, &cmap_root, &mut snapshot)?;
    Ok(snapshot)
}

fn collect_snapshot(
    root: &Path,
    current: &Path,
    snapshot: &mut BTreeMap<String, String>,
) -> std::io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(current)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_snapshot(root, &path, snapshot)?;
            continue;
        }

        let relative = path
            .strip_prefix(root)
            .expect("snapshot file should be within .cmap")
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(&path)?;
        snapshot.insert(relative, blake3::hash(&bytes).to_hex().to_string());
    }

    Ok(())
}

fn parse_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("scan output should be valid JSON")
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

fn run_cmap_output(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cmap"))
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .expect("execute cmap command")
}
