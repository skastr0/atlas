import { describe, expect, test } from "bun:test";

import {
  buildAtlasPromptPart,
  getSessionId,
  getWorkspaceRoot,
  shouldRunChangedOnlyBuild,
} from "../src/index.ts";
import { promptWithSessionContext } from "../src/session-prompt.ts";

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

  test("injects atlas context as a prompt part for separate synthetic prompts", () => {
    expect(buildAtlasPromptPart("Root facts")).toEqual({
      type: "text",
      text: "[System] Knowledge Base Atlas for this repository:\n\nRoot facts",
      synthetic: true,
    });
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
