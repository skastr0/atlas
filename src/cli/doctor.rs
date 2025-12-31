//! `cmap doctor` command - Report issues

use crate::LogLevel;
use anyhow::Result;
use std::path::Path;

pub fn run(root: &Path, log_level: LogLevel) -> Result<()> {
    if log_level != LogLevel::Quiet {
        println!("Checking {} for issues...", root.display());
    }

    // TODO: Implement diagnostics
    // 1. Check for extraction failures
    // 2. Check for stale cache entries
    // 3. Check for potential duplicates
    // 4. Check for missing pdftotext

    if log_level != LogLevel::Quiet {
        println!("(doctor not yet implemented)");
    }

    Ok(())
}
