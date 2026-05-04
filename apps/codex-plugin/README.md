# ContextMap Codex plugin

This package is the Codex equivalent of the OpenCode ContextMap plugin.

Codex currently exposes installable plugin bundles and lifecycle command hooks, not a long-lived TypeScript server plugin API like OpenCode. This plugin therefore implements the closest supported behavior:

- `SessionStart` for `startup`, `resume`, and `clear` runs `cmap init --root "$root" --quiet` when `cmap` is on `PATH`.
- `PostToolUse` for Codex edit tools (`apply_patch`, `Edit`, `Write`) runs `cmap build --root "$root" --changed-only` when `cmap` is on `PATH` and `.cmap` exists.
- `$root` is the git worktree root when `git rev-parse --show-toplevel` succeeds, otherwise the Codex session cwd.

The OpenCode plugin also debounces file-change events in-process. Codex lifecycle hooks are command invocations, so this plugin refreshes after each supported edit hook instead of maintaining a debounced background fiber.

## Local development

```bash
node --test apps/codex-plugin/test/hooks.test.mjs
```

## Install from the repo marketplace

This repository includes `.agents/plugins/marketplace.json`, which points at `apps/codex-plugin`.

After adding or updating the local marketplace in Codex, restart Codex and install the `context-map` plugin from the plugin browser. Codex installs local plugins into its plugin cache, so restart Codex after changing this package during development.

Codex hooks must be enabled in the active config:

```toml
[features]
codex_hooks = true
```
