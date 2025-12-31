//! context-map: Deterministic knowledge base indexer for AI agents
//!
//! Generates multi-resolution markdown indexes of knowledge bases,
//! solving the "AI doesn't know what it knows" problem through
//! cheap, deterministic, static analysis.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod aggregate;
mod analyze;
mod cache;
mod cli;
mod config;
mod extract;
mod render;
mod scan;
mod types;

// Re-export for use in cli modules
pub use aggregate::*;
pub use analyze::*;
pub use cache::*;
pub use config::*;
pub use extract::*;
pub use render::*;
pub use scan::*;
pub use types::*;

#[derive(Parser)]
#[command(name = "cmap")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Knowledge base root directory
    #[arg(short, long, global = true, default_value = ".")]
    root: PathBuf,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Suppress output
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize .cmap in current directory
    Init,

    /// Scan files and update fingerprints
    Scan {
        /// Show what would be scanned without writing
        #[arg(long)]
        dry_run: bool,
    },

    /// Build/update index and generate views
    Build {
        /// Only process changed files
        #[arg(long)]
        changed_only: bool,

        /// Force full rebuild, ignoring cache
        #[arg(long)]
        force: bool,
    },

    /// Report issues (extraction failures, stale cache, duplicates)
    Doctor,

    /// Remove all cached data
    Clean {
        /// Also remove generated views
        #[arg(long)]
        all: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging based on verbosity
    let log_level = match (cli.quiet, cli.verbose) {
        (true, _) => LogLevel::Quiet,
        (_, 0) => LogLevel::Normal,
        (_, 1) => LogLevel::Verbose,
        (_, _) => LogLevel::Debug,
    };

    match cli.command {
        Commands::Init => cli::init::run(&cli.root, log_level),
        Commands::Scan { dry_run } => cli::scan::run(&cli.root, dry_run, log_level),
        Commands::Build {
            changed_only,
            force,
        } => cli::build::run(&cli.root, changed_only, force, log_level),
        Commands::Doctor => cli::doctor::run(&cli.root, log_level),
        Commands::Clean { all } => cli::clean::run(&cli.root, all, log_level),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Quiet,
    Normal,
    Verbose,
    Debug,
}

impl LogLevel {
    pub fn should_print(&self, level: LogLevel) -> bool {
        (*self as u8) >= (level as u8)
    }
}
