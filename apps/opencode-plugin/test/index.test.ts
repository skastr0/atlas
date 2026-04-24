import { describe, expect, test } from "bun:test";

import {
  getSessionId,
  getWorkspaceRoot,
  shouldRunChangedOnlyBuild,
} from "../src/index.ts";

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
});
