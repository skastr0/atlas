# Security Policy

## Supported Status

atlas is experimental and solo-maintained. Security reports are reviewed on a best-effort basis, without a formal response SLA.

## Reporting A Vulnerability

Please do not open a public issue for suspected vulnerabilities. Report privately to the maintainer through GitHub's private vulnerability reporting for this repository, or contact the maintainer directly if that is not yet enabled.

Include:

- affected version or commit
- reproduction steps
- impact
- relevant logs or proof of concept

Please redact tokens, personal data, private endpoints, and unrelated secrets from reports.

## Scope

Runnable CLIs, plugins, package installation paths, generated release assets, and documented local workflows are in scope. Third-party services, user-provided credentials, and local machine configuration outside this repository are out of scope unless this project directly mishandles them.

## Supply Chain Notes

There are no official crates.io packages, npm packages, GitHub release assets, or Homebrew formulas for atlas yet. Do not trust install commands, packages, or binaries that claim to distribute this project unless they are listed in the repository README after a public release.
