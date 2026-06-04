# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning for its declared public API.

## [Unreleased]

### Changed
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
