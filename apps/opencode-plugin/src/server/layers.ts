import type { PluginInput } from "@opencode-ai/plugin";
import { Context, Layer } from "effect";
import { makeServerLoggerLayer, PluginLogger } from "../shared/logger";

export type ShellTextRunner = PluginInput["$"];
export type OpencodeClient = PluginInput["client"];

export class CmapPluginContext extends Context.Tag(
  "@context-map/opencode-plugin/CmapPluginContext",
)<
  CmapPluginContext,
  {
    readonly client: OpencodeClient;
    readonly $: ShellTextRunner;
    readonly directory: string;
    readonly worktree: string;
    readonly root: string;
  }
>() {}

export type ServerRuntimeEnv = CmapPluginContext | PluginLogger;

export const getWorkspaceRoot = (worktree: string, directory: string): string =>
  worktree || directory;

export const makeServerLayer = (input: PluginInput) => {
  const root = getWorkspaceRoot(input.worktree, input.directory);

  return Layer.mergeAll(
    Layer.succeed(CmapPluginContext, {
      client: input.client,
      $: input.$,
      directory: input.directory,
      worktree: input.worktree,
      root,
    }),
    makeServerLoggerLayer(input.client),
  );
};
