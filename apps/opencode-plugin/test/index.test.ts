import type { PluginInput } from "@opencode-ai/plugin";
import { createOpencodeClient } from "@opencode-ai/sdk";
import { describe, expect, test } from "bun:test";
import { $ } from "bun";
import { mkdtemp } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

import {
  AtlasOpenCodePlugin,
  getSessionId,
  getWorkspaceRoot,
  shouldRunChangedOnlyBuild,
  shouldRunAtlasInit,
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

const makePluginInputWithLogs = (
  root: string,
  logs: Array<{ level: string; message: string }>,
  shell: PluginInput["$"],
): PluginInput => ({
  ...makePluginInput(root),
  client: {
    app: {
      log: async ({ body }: { body: { level: string; message: string } }) => {
        logs.push(body);
      },
    },
  } as unknown as PluginInput["client"],
  $: shell,
});

const makeMissingAtlasShell = (): PluginInput["$"] => {
  const shell = ((strings: TemplateStringsArray, ...expressions: unknown[]) => {
    const command = strings.reduce(
      (acc, part, index) => `${acc}${part}${index < expressions.length ? String(expressions[index]) : ""}`,
      "",
    );
    return {
      text: async () => {
        if (command.includes("command -v atlas")) {
          throw new Error("atlas is not on PATH");
        }
        if (command.includes("atlas ")) {
          throw new Error(`unexpected atlas invocation: ${command}`);
        }
        return "";
      },
    };
  }) as unknown as PluginInput["$"];

  shell.braces = () => [];
  shell.escape = (input: string) => input;
  shell.env = () => shell;
  shell.cwd = () => shell;
  shell.nothrow = () => shell;
  shell.throws = () => shell;

  return shell;
};

describe("atlas OpenCode hook helpers", () => {
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

  test("runs atlas init on startup and session lifecycle events", () => {
    expect(shouldRunAtlasInit("server.connected")).toBe(true);
    expect(shouldRunAtlasInit("session.created")).toBe(true);
    expect(shouldRunAtlasInit("file.edited")).toBe(false);
  });

  test("does not register atlas/session prompt injection hooks", async () => {
    const root = await mkdtemp(join(tmpdir(), "atlas-opencode-plugin-"));
    const hooks = await AtlasOpenCodePlugin(makePluginInput(root));

    expect(Reflect.has(hooks, "chat.message")).toBe(false);
    expect(Reflect.has(hooks, "experimental.chat.system.transform")).toBe(false);
    expect(Reflect.has(hooks, "experimental.session.compacting")).toBe(false);
  });

  test("does not register compaction context mutation", async () => {
    const root = await mkdtemp(join(tmpdir(), "atlas-opencode-plugin-"));
    const hooks = await AtlasOpenCodePlugin(makePluginInput(root));

    expect(Reflect.has(hooks, "experimental.session.compacting")).toBe(false);
  });

  test("does not log atlas command failures when the CLI is unavailable", async () => {
    const root = await mkdtemp(join(tmpdir(), "atlas-opencode-plugin-"));
    const logs: Array<{ level: string; message: string }> = [];
    const hooks = await AtlasOpenCodePlugin(
      makePluginInputWithLogs(root, logs, makeMissingAtlasShell()),
    );

    await hooks.event?.({ event: { type: "server.connected", properties: {} } } as Parameters<
      NonNullable<typeof hooks.event>
    >[0]);

    expect(logs.some((entry) => entry.message === "atlas init failed")).toBe(false);
  });
});
