# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning for its declared public API.

## [Unreleased]

## [0.2.0] - 2026-09-25

### Changed
- `cli::search::run` now takes a `SearchRequest` struct instead of individual query, filter, and output arguments. The `atlas search` CLI is unchanged.
- CI now gates on `cargo fmt`, strict clippy (`-D warnings`), and both plugin test suites.

### Fixed
- Resolved all clippy warnings on current stable Rust.

## [0.1.2] - 2026-06-04

### Added
- Documented Codex marketplace install commands for local and Git-backed Atlas plugin installs.

### Changed
- Aligned the Codex plugin manifest version with the published package version.
- Switched npm plugin publishing to the same `v*` release tags as the Cargo crate so package versions stay aligned.
- Documented the Atlas publishing and versioning strategy, including the plugin dependency on the CLI.

## [0.1.1] - 2026-06-04

### Added
- Configured crates.io trusted publishing for the protected release workflow.
- Published the Atlas Codex and OpenCode plugins to npm with trusted publishing configured.

### Fixed
- Made the Atlas OpenCode plugin no-op when the `atlas` CLI is unavailable.

## [0.1.0] - 2026-06-03

### Added
- Initial experimental release of the deterministic knowledge base indexer.
- Published the official crates.io package under the name `agent-atlas`, with the installed binary kept as `atlas`.
- CLI commands for `init`, `scan`, `build`, `search`, `doctor`, and `clean`.
- Markdown, plaintext, PDF, Rust, TypeScript, JavaScript, and common config/text extraction.
- Deterministic atlas, folder index, term index, and graph view generation.
