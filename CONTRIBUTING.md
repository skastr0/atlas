# Contributing

Thanks for helping improve atlas. This is an experimental, solo-maintained project. The default path is issues first, especially for anything beyond a small bug fix or documentation correction.

## What Helps

- Reproducible bug reports
- Small fixes with tests or clear verification notes
- Documentation corrections
- Scoped proposals discussed before a large implementation

## What Is Out Of Scope

- Large rewrites without prior discussion
- Broad feature work that changes the maintenance burden substantially
- Public security reports; use the private reporting path in SECURITY.md

## Local Workflow

Use the commands documented in README.md for this repository. Before opening a pull request, run the available verification commands and include the results in the PR description.

```bash
cargo test --all-features
cargo package --list
```

## Pull Requests

Small pull requests for clear bugs, documentation corrections, or agreed follow-ups are easiest to review.

Before opening a larger pull request:

1. Open an issue.
2. Wait for maintainer confirmation that the change fits the project.
3. Keep the implementation scoped to the accepted behavior.

Unsolicited rewrites, new subsystems, broad formatting changes, generated churn, or unrelated dependency updates may be closed without review. A PR may also be declined for scope, maintenance cost, compatibility risk, or product direction even when the implementation is technically sound.

By contributing, you agree that your contribution is licensed under the MIT license used by this project.

## Conduct

Be direct, specific, and respectful. Issues or pull requests may be closed when they are hostile, off-topic, spammy, or outside the project's stated scope.
