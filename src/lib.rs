//! atlas library
//!
//! Core types and functionality for knowledge base indexing.

pub mod aggregate;
pub mod analyze;
pub mod cache;
pub mod cli;
pub mod config;
pub mod extract;
pub mod render;
pub mod scan;
pub mod types;

pub use aggregate::*;
pub use analyze::*;
pub use cache::*;
pub use config::Config;
pub use extract::*;
pub use render::*;
pub use scan::*;
pub use types::*;

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
