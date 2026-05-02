import { describe, expect, test } from "bun:test";
import { mkdir, mkdtemp, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

import {
  buildAtlasPromptPart,
  buildAtlasSystemContext,
  CmapOpenCodePlugin,
  getSessionId,
  getWorkspaceRoot,
  shouldRefreshAtlasContext,
  shouldRunChangedOnlyBuild,
  shouldRunCmapInit,
} from "../src/index.ts";
import { promptWithSessionContext } from "../src/session-prompt.ts";

const makePluginInput = (root: string, clientOverrides: Record<string, unknown> = {}) =>
  ({
    directory: root,
    worktree: root,
    client: {
      app: {
        log: async () => undefined,
      },
      ...clientOverrides,
    },
    $: (() => {
      throw new Error("shell should not be used by this test");
    }) as never,
  }) as never;

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

  test("refreshes atlas context only at session boundaries", () => {
    expect(shouldRefreshAtlasContext("session.created")).toBe(true);
    expect(shouldRefreshAtlasContext("session.compacted")).toBe(true);
    expect(shouldRefreshAtlasContext("message.updated")).toBe(false);
  });

  test("injects atlas context as a prompt part for separate synthetic prompts", () => {
    expect(buildAtlasPromptPart("Root facts")).toEqual({
      type: "text",
      text: "[System] Knowledge Base Atlas for this repository:\n\nRoot facts",
      synthetic: true,
    });
  });

  test("does not register chat.message atlas injection", async () => {
    const root = await mkdtemp(join(tmpdir(), "cmap-opencode-plugin-"));
    const hooks = await CmapOpenCodePlugin(makePluginInput(root));

    expect(Reflect.has(hooks, "chat.message")).toBe(false);
    expect(typeof hooks["experimental.chat.system.transform"]).toBe("function");
    expect(Reflect.has(hooks, "experimental.session.compacting")).toBe(false);
  });

  test("injects ROOT_ATLAS.md through system transform instead of message submit", async () => {
    const root = await mkdtemp(join(tmpdir(), "cmap-opencode-plugin-"));
    await mkdir(join(root, ".cmap", "views"), { recursive: true });
    await writeFile(join(root, ".cmap", "views", "ROOT_ATLAS.md"), "Root facts");

    const promptCalls: unknown[] = [];
    const hooks = await CmapOpenCodePlugin(
      makePluginInput(root, {
        session: {
          prompt: async (options: unknown) => {
            promptCalls.push(options);
          },
        },
      }),
    );
    const output = { system: ["base"] };

    await hooks["experimental.chat.system.transform"]?.(
      { sessionID: "ses_123", model: { providerID: "openai", modelID: "gpt-5.5" } } as never,
      output,
    );

    expect(output.system).toEqual(["base", buildAtlasSystemContext("Root facts")]);
    expect(promptCalls).toEqual([]);
  });

  test("does not mutate system prompt or submit a message when ROOT_ATLAS.md is absent", async () => {
    const root = await mkdtemp(join(tmpdir(), "cmap-opencode-plugin-"));
    const promptCalls: unknown[] = [];

    const hooks = await CmapOpenCodePlugin(
      makePluginInput(root, {
        session: {
          prompt: async (options: unknown) => {
            promptCalls.push(options);
          },
        },
      }),
    );
    const output = { system: ["base"] };

    await hooks["experimental.chat.system.transform"]?.(
      { sessionID: "ses_123", model: { providerID: "openai", modelID: "gpt-5.5" } } as never,
      output,
    );

    expect(output.system).toEqual(["base"]);
    expect(promptCalls).toEqual([]);
  });

  test("does not register compaction context mutation", async () => {
    const root = await mkdtemp(join(tmpdir(), "cmap-opencode-plugin-"));
    const hooks = await CmapOpenCodePlugin(makePluginInput(root));

    expect(Reflect.has(hooks, "experimental.session.compacting")).toBe(false);
  });

  test("safe prompt helper preserves latest user agent and model context", async () => {
    const promptCalls: unknown[] = [];
    const client = {
      session: {
        messages: async () => ({
          data: [
            {
              info: {
                role: "user",
                agent: "orchestrator-engineer",
                model: { providerID: "openai", modelID: "gpt-5.5", variant: "high" },
              },
            },
          ],
        }),
        prompt: async (options: unknown) => {
          promptCalls.push(options);
        },
      },
    };

    await promptWithSessionContext(client, "ses_123", {
      noReply: true,
      parts: [buildAtlasPromptPart("Root facts")],
    });

    expect(promptCalls).toEqual([
      {
        path: { id: "ses_123" },
        body: {
          agent: "orchestrator-engineer",
          model: { providerID: "openai", modelID: "gpt-5.5" },
          variant: "high",
          noReply: true,
          parts: [buildAtlasPromptPart("Root facts")],
        },
      },
    ]);
  });
});
