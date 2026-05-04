import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";
import { test } from "node:test";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const pluginRoot = new URL("..", import.meta.url);
const hooksPath = new URL("../hooks/hooks.json", import.meta.url);
const manifestPath = new URL("../.codex-plugin/plugin.json", import.meta.url);
const marketplacePath = new URL("../../../.agents/plugins/marketplace.json", import.meta.url);

const readJson = async (url) => JSON.parse(await readFile(url, "utf8"));

const getCommand = (hooksConfig, eventName) =>
  hooksConfig.hooks[eventName][0].hooks[0].command;

const runHookCommand = async (command, cwd, binDir) =>
  execFileAsync("sh", ["-c", command], {
    cwd,
    env: {
      ...process.env,
      PATH: `${binDir}${delimiter}${process.env.PATH}`,
    },
  });

const makeFakeCmap = async (dir) => {
  const binDir = join(dir, "bin");
  const logPath = join(dir, "cmap.log");
  await mkdir(binDir);
  await writeFile(
    join(binDir, "cmap"),
    `#!/bin/sh
printf '%s\\n' "$*" >> "${logPath}"
`,
    { mode: 0o755 },
  );
  return { binDir, logPath };
};

test("manifest points at bundled hooks with plugin-root relative paths", async () => {
  const manifest = await readJson(manifestPath);

  assert.equal(manifest.name, "context-map");
  assert.equal(manifest.hooks, "./hooks/hooks.json");
  assert.ok(manifest.interface.displayName);
});

test("hooks wire Codex session and edit events", async () => {
  const hooks = await readJson(hooksPath);

  assert.equal(hooks.hooks.SessionStart[0].matcher, "startup|resume|clear");
  assert.equal(hooks.hooks.PostToolUse[0].matcher, "^apply_patch$|^Edit$|^Write$");
  assert.match(getCommand(hooks, "SessionStart"), /cmap init --root "\$root" --quiet/);
  assert.match(getCommand(hooks, "PostToolUse"), /cmap build --root "\$root" --changed-only/);
});

test("post-tool matcher ignores non-edit tools", async () => {
  const hooks = await readJson(hooksPath);
  const matcher = new RegExp(hooks.hooks.PostToolUse[0].matcher);

  assert.equal(matcher.test("apply_patch"), true);
  assert.equal(matcher.test("Edit"), true);
  assert.equal(matcher.test("Write"), true);
  assert.equal(matcher.test("Bash"), false);
  assert.equal(matcher.test("mcp__filesystem__read_file"), false);
});

test("repo marketplace exposes the local plugin package", async () => {
  const marketplace = await readJson(marketplacePath);
  const plugin = marketplace.plugins.find((entry) => entry.name === "context-map");

  assert.ok(plugin);
  assert.equal(plugin.source.source, "local");
  assert.equal(plugin.source.path, "./apps/codex-plugin");
  assert.equal(plugin.policy.installation, "AVAILABLE");
});

test("session start initializes ContextMap at the git root", async () => {
  const temp = await mkdtemp(join(tmpdir(), "cmap-codex-plugin-"));
  try {
    const { binDir, logPath } = await makeFakeCmap(temp);
    const repo = join(temp, "repo");
    const child = join(repo, "docs");
    await mkdir(child, { recursive: true });
    await execFileAsync("git", ["init"], { cwd: repo });

    const hooks = await readJson(hooksPath);
    await runHookCommand(getCommand(hooks, "SessionStart"), child, binDir);

    const canonicalRepo = await realpath(repo);
    assert.equal(
      (await readFile(logPath, "utf8")).trim(),
      `init --root ${canonicalRepo} --quiet`,
    );
  } finally {
    await rm(temp, { recursive: true, force: true });
  }
});

test("edit refresh runs changed-only build when .cmap exists", async () => {
  const temp = await mkdtemp(join(tmpdir(), "cmap-codex-plugin-"));
  try {
    const { binDir, logPath } = await makeFakeCmap(temp);
    const repo = join(temp, "repo");
    await mkdir(join(repo, ".cmap"), { recursive: true });
    await execFileAsync("git", ["init"], { cwd: repo });

    const hooks = await readJson(hooksPath);
    await runHookCommand(getCommand(hooks, "PostToolUse"), repo, binDir);

    const canonicalRepo = await realpath(repo);
    assert.equal(
      (await readFile(logPath, "utf8")).trim(),
      `build --root ${canonicalRepo} --changed-only`,
    );
  } finally {
    await rm(temp, { recursive: true, force: true });
  }
});

test("edit refresh is a no-op before ContextMap is initialized", async () => {
  const temp = await mkdtemp(join(tmpdir(), "cmap-codex-plugin-"));
  try {
    const { binDir, logPath } = await makeFakeCmap(temp);
    const repo = join(temp, "repo");
    await mkdir(repo);

    const hooks = await readJson(hooksPath);
    await runHookCommand(getCommand(hooks, "PostToolUse"), repo, binDir);

    await assert.rejects(readFile(logPath, "utf8"), { code: "ENOENT" });
  } finally {
    await rm(temp, { recursive: true, force: true });
  }
});

test("plugin root is the expected fixture location", () => {
  assert.equal(pluginRoot.pathname.endsWith("/apps/codex-plugin/"), true);
});
