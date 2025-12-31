//! `cmap scan` command - Scan files and update fingerprints

use crate::LogLevel;
use anyhow::Result;
use std::path::Path;

pub fn run(root: &Path, dry_run: bool, log_level: LogLevel) -> Result<()> {
    if log_level != LogLevel::Quiet {
        println!(
            "Scanning {}{}",
            root.display(),
            if dry_run { " (dry run)" } else { "" }
        );
    }

    // TODO: Implement scanning
    // 1. Walk directory tree
    // 2. Apply ignore patterns
    // 3. Compute fingerprints (mtime + size)
    // 4. Compare with cached fingerprints
    // 5. Report new/modified/deleted files

    if log_level != LogLevel::Quiet {
        println!("(scan not yet implemented)");
    }

    Ok(())
}
