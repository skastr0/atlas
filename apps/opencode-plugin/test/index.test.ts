import type { PluginInput } from "@opencode-ai/plugin";
import { createOpencodeClient } from "@opencode-ai/sdk";
import { describe, expect, test } from "bun:test";
import { $ } from "bun";
import { mkdtemp } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

import {
  CmapOpenCodePlugin,
  getSessionId,
  getWorkspaceRoot,
  shouldRunChangedOnlyBuild,
  shouldRunCmapInit,
} from "../src/index.ts";

const makePluginInput = (root: string): PluginInput => ({
  directory: root,
  worktree: root,
  serverUrl: new URL("http://127.0.0.1:0"),
  project: {
    id: "test-project",
    worktree: root,
    time: { created: 0 },
  },
  client: createOpencodeClient({ baseUrl: "http://127.0.0.1:0" }),
  $,
});

describe("context-map OpenCode hook helpers", () => {
  test("uses the git worktree when OpenCode provides one", () => {
    expect(getWorkspaceRoot("/repo", "/repo/subdir")).toBe("/repo");
    expect(getWorkspaceRoot("", "/repo/subdir")).toBe("/repo/subdir");
  });

  test("reads sessionID from the current OpenCode event shape", () => {
    expect(getSessionId({ properties: { sessionID: "ses_123" } })).toBe("ses_123");
    expect(getSessionId({ properties: { id: "legacy" } })).toBeUndefined();
    expect(getSessionId({ properties: null })).toBeUndefined();
  });

  test("limits rebuilds to file change events", () => {
    expect(shouldRunChangedOnlyBuild("file.edited")).toBe(true);
    expect(shouldRunChangedOnlyBuild("file.watcher.updated")).toBe(true);
    expect(shouldRunChangedOnlyBuild("message.updated")).toBe(false);
  });

  test("runs cmap init on startup and session lifecycle events", () => {
    expect(shouldRunCmapInit("server.connected")).toBe(true);
    expect(shouldRunCmapInit("session.created")).toBe(true);
    expect(shouldRunCmapInit("file.edited")).toBe(false);
  });

  test("does not register atlas/session prompt injection hooks", async () => {
    const root = await mkdtemp(join(tmpdir(), "cmap-opencode-plugin-"));
    const hooks = await CmapOpenCodePlugin(makePluginInput(root));

    expect(Reflect.has(hooks, "chat.message")).toBe(false);
    expect(Reflect.has(hooks, "experimental.chat.system.transform")).toBe(false);
    expect(Reflect.has(hooks, "experimental.session.compacting")).toBe(false);
  });

  test("does not register compaction context mutation", async () => {
    const root = await mkdtemp(join(tmpdir(), "cmap-opencode-plugin-"));
    const hooks = await CmapOpenCodePlugin(makePluginInput(root));

    expect(Reflect.has(hooks, "experimental.session.compacting")).toBe(false);
  });
});
