# @skastr0/atlas-opencode-plugin

OpenCode plugin for Atlas.

The plugin initializes Atlas when OpenCode connects or creates a session, then debounces changed-only Atlas rebuilds after file edit events. It expects the `atlas` binary to be available on `PATH`; install it with `cargo install agent-atlas`.

For normal use, install the same version of the `agent-atlas` CLI crate and this plugin package.

## Build

```sh
bun run --cwd apps/opencode-plugin build
```

The build emits the OpenCode plugin entrypoint at `apps/opencode-plugin/dist/index.js`.
