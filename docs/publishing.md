# Publishing and versioning

Atlas publishes one CLI crate and two agent plugin packages:

- `agent-atlas` on crates.io, which installs the `atlas` binary.
- `@skastr0/atlas-codex-plugin` on npm.
- `@skastr0/atlas-opencode-plugin` on npm.

## Version strategy

All published Atlas packages use the same SemVer version.

This is intentional. The Codex and OpenCode plugins shell out to the `atlas` CLI on `PATH`; they do not bundle the CLI. Keeping the crate and plugin package versions aligned makes support and install instructions simple:

```bash
cargo install agent-atlas --version <version>
npm install @skastr0/atlas-codex-plugin@<version>
npm install @skastr0/atlas-opencode-plugin@<version>
```

If a release only changes one artifact, still bump and publish all three artifacts with the same version. A no-op crate or plugin release is preferable to version drift where plugin `0.1.7` has to be matched against CLI `0.1.1`.

## Plugin dependency on the CLI

Both plugins require the `atlas` binary to be available on `PATH`.

- The Codex plugin checks for `atlas` before running hook commands and exits successfully when the CLI is unavailable.
- The OpenCode plugin checks for `atlas` before running event hooks and no-ops when the CLI is unavailable.

The plugins should fail soft when the CLI is missing. They should not block an agent session just because Atlas is not installed yet.

For normal use, install the same version of the CLI and plugin package. If a compatibility exception is ever needed, document it in `CHANGELOG.md` and the affected plugin README before publishing.

## Release topology

Atlas uses one release tag for all published artifacts:

```bash
v<version>
```

The same `v*` tag triggers:

- `.github/workflows/release-crate.yml` for crates.io.
- `.github/workflows/npm-plugins-publish.yml` for npm plugins.

Both workflows use the protected GitHub `release` environment. The environment is restricted to `v*` tags and requires maintainer approval before publishing.

Do not use separate plugin release tags. In particular, do not reintroduce `plugins-v*` unless the project deliberately chooses independent plugin versioning.

## Authentication model

Publishing is CI-owned after the initial bootstrap.

- crates.io uses trusted publishing through `rust-lang/crates-io-auth-action`.
- npm uses trusted publishing through GitHub Actions OIDC.
- The release workflows should not use long-lived npm or crates.io publish tokens.
- Local publish is only for explicitly approved bootstrap or emergency exceptions.

Because crates.io is configured to require trusted publishing for new versions, local `cargo publish` with an API token should be rejected for normal releases.

## Release checklist

Before tagging a release:

1. Bump `Cargo.toml`.
2. Bump `Cargo.lock`.
3. Bump `apps/codex-plugin/package.json`.
4. Bump `apps/opencode-plugin/package.json`.
5. Update `CHANGELOG.md`.
6. Verify the version is not already published:

```bash
cargo search agent-atlas --limit 1
npm view @skastr0/atlas-codex-plugin@<version> version --prefer-online
npm view @skastr0/atlas-opencode-plugin@<version> version --prefer-online
```

The npm version checks should return 404 before publishing.

Run local verification:

```bash
cargo test --all-features
cargo package --list
cargo publish --dry-run --locked
npm --prefix apps/codex-plugin test
bun run --cwd apps/opencode-plugin verify
```

Then publish through CI:

```bash
git add CHANGELOG.md Cargo.toml Cargo.lock apps/codex-plugin/package.json apps/opencode-plugin/package.json
git commit -m "chore: prepare <version> release"
git push origin main
git tag -a v<version> -m "atlas <version>"
git push origin v<version>
```

Approve the `release` environment for both publish workflows.

## Post-publish verification

Verify the live packages from a clean temp directory:

```bash
cargo install agent-atlas --version <version> --root /tmp/atlas-release-smoke --force
/tmp/atlas-release-smoke/bin/atlas --version
npm install @skastr0/atlas-codex-plugin@<version> @skastr0/atlas-opencode-plugin@<version>
node -e "import('@skastr0/atlas-opencode-plugin').then((mod) => console.log(typeof mod.default.server))"
```

Also check npm provenance:

```bash
npm view @skastr0/atlas-codex-plugin@<version> dist --json --prefer-online
npm view @skastr0/atlas-opencode-plugin@<version> dist --json --prefer-online
```

Each npm package should include an `attestations.provenance` entry.

## Failure handling

Published versions are immutable.

- For crates.io, publish a fix version or yank a broken version with `cargo yank`.
- For npm, publish a fix version or deprecate a broken version.
- Do not try to overwrite an existing version.

If a release workflow fails after one artifact has already published, first verify registry state. Rerun only workflows whose artifacts are still unpublished. The npm workflow has skip-existing-version logic, but the Cargo workflow should not be rerun once the crate version is live. If the release state is ambiguous, prepare a new version and tag instead of trying to overwrite an existing version.
