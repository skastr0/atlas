use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

#[derive(Debug, Clone)]
struct ArtifactSnapshot {
    hash: String,
    content: String,
}

#[test]
fn build_outputs_are_identical_across_two_force_runs() {
    let fixture_root = TempDir::new().expect("create temp corpus root");
    write_tie_fixture(fixture_root.path()).expect("write tie fixture corpus");

    run_atlas(fixture_root.path(), &["init"]);
    configure_repro_config(fixture_root.path()).expect("configure deterministic repro settings");

    run_atlas(fixture_root.path(), &["build", "--force"]);
    let first_run = collect_artifacts(fixture_root.path()).expect("collect first run artifacts");

    run_atlas(fixture_root.path(), &["build", "--force"]);
    let second_run = collect_artifacts(fixture_root.path()).expect("collect second run artifacts");

    assert_artifacts_match(&first_run, &second_run);
}

fn run_atlas(root: &Path, args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .arg("--quiet")
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .expect("execute atlas command");

    assert!(
        output.status.success(),
        "atlas command failed: {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_tie_fixture(root: &Path) -> std::io::Result<()> {
    write_file(
        root.join("alpha/tie-a.md"),
        "# The And\n\nanchor beacon anchor beacon anchor beacon.\n",
    )?;
    write_file(
        root.join("alpha/tie-b.md"),
        "# The And\n\nanchor beacon anchor beacon anchor beacon!\n",
    )?;
    write_file(
        root.join("beta/tie-c.md"),
        "# The And\n\nanchor beacon anchor beacon anchor beacon?\n",
    )?;
    write_file(
        root.join("beta/tie-d.md"),
        "# The And\n\nanchor beacon anchor beacon anchor beacon;\n",
    )?;

    Ok(())
}

fn configure_repro_config(root: &Path) -> std::io::Result<()> {
    let config_path = root.join(".atlas/config.toml");
    let config = fs::read_to_string(&config_path)?;
    let updated = config.replace("top_phrases = 10", "top_phrases = 0");
    fs::write(config_path, updated)
}

fn write_file(path: PathBuf, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

fn collect_artifacts(root: &Path) -> std::io::Result<BTreeMap<String, ArtifactSnapshot>> {
    let atlas = root.join(".atlas");
    let mut artifact_paths = vec![
        PathBuf::from("last-build.json"),
        PathBuf::from("views/ROOT_ATLAS.md"),
        PathBuf::from("views/TERMS.md"),
        PathBuf::from("views/CONNECTIONS.md"),
        PathBuf::from("global/term_index.json"),
    ];

    let folders_dir = atlas.join("views/folders");
    let mut folder_artifacts: Vec<PathBuf> = fs::read_dir(&folders_dir)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                Some(PathBuf::from("views/folders").join(entry.file_name()))
            } else {
                None
            }
        })
        .collect();
    folder_artifacts.sort();

    assert!(
        !folder_artifacts.is_empty(),
        "expected folder artifacts under {}",
        folders_dir.display()
    );

    artifact_paths.extend(folder_artifacts);
    artifact_paths.sort();

    let mut snapshots = BTreeMap::new();
    for relative in artifact_paths {
        let full_path = atlas.join(&relative);
        let content = fs::read_to_string(&full_path)?;
        let hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        let key = relative.to_string_lossy().replace('\\', "/");
        snapshots.insert(key, ArtifactSnapshot { hash, content });
    }

    Ok(snapshots)
}

fn assert_artifacts_match(
    first: &BTreeMap<String, ArtifactSnapshot>,
    second: &BTreeMap<String, ArtifactSnapshot>,
) {
    let first_paths: Vec<String> = first.keys().cloned().collect();
    let second_paths: Vec<String> = second.keys().cloned().collect();
    assert_eq!(
        first_paths, second_paths,
        "artifact path set differs between runs"
    );

    let mut mismatches = Vec::new();
    for (path, run_one) in first {
        let run_two = second
            .get(path)
            .expect("artifact should exist in both runs");
        if run_one.content != run_two.content {
            mismatches.push(format!(
                "- {}: run1={} run2={}",
                path, run_one.hash, run_two.hash
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "artifact reproducibility mismatch:\n{}",
        mismatches.join("\n")
    );
}
