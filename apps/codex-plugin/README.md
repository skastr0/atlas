# Atlas Codex plugin

This package is the Codex equivalent of the OpenCode Atlas plugin.

Install the `atlas` CLI separately with `cargo install agent-atlas`. For normal use, install the same version of the CLI and this plugin package.

Codex currently exposes installable plugin bundles and lifecycle command hooks, not a long-lived TypeScript server plugin API like OpenCode. This plugin therefore implements the closest supported behavior:

- `SessionStart` for `startup`, `resume`, and `clear` runs `atlas init --root "$root" --quiet` when `atlas` is on `PATH`.
- `PostToolUse` for Codex edit tools (`apply_patch`, `Edit`, `Write`) runs `atlas build --root "$root" --changed-only` when `atlas` is on `PATH` and `.atlas` exists.
- `$root` is the git worktree root when `git rev-parse --show-toplevel` succeeds, otherwise the Codex session cwd.

The OpenCode plugin also debounces file-change events in-process. Codex lifecycle hooks are command invocations, so this plugin refreshes after each supported edit hook instead of maintaining a debounced background fiber.

## Local development

```bash
node --test apps/codex-plugin/test/hooks.test.mjs
```

## Install from the repo marketplace

This repository includes `.agents/plugins/marketplace.json`, which points at `apps/codex-plugin`.

For a local checkout:

```sh
codex plugin marketplace add /path/to/atlas
codex plugin add atlas@atlas-local
```

For a Git-backed marketplace snapshot:

```sh
codex plugin marketplace add skastr0/atlas --ref main
codex plugin add atlas@atlas-local
```

After adding or updating the marketplace in Codex, restart Codex and install the `atlas` plugin from the plugin browser or CLI. Codex installs local plugins into its plugin cache, so restart Codex after changing this package during development.

Codex hooks must be enabled in the active config:

```toml
[features]
codex_hooks = true
```
