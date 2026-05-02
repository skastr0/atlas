import type { Hooks, Plugin, PluginModule } from "@opencode-ai/plugin";
import { Effect, ManagedRuntime } from "effect";
import { injectAtlasIntoSystem, onEvent } from "./server/hooks";
import { getWorkspaceRoot, makeServerLayer, type ServerRuntimeEnv } from "./server/layers";
import { PLUGIN_ID } from "./shared/constants";
import { toThrowable } from "./shared/errors";
import { PluginLogger } from "./shared/logger";

export { buildAtlasPromptPart, buildAtlasSystemContext } from "./server/atlas";
export {
  getSessionId,
  shouldRefreshAtlasContext,
  shouldRunChangedOnlyBuild,
  shouldRunCmapInit,
} from "./server/hooks";
export { getWorkspaceRoot } from "./server/layers";

export const CmapOpenCodePlugin: Plugin = async (input) => {
  const runtime = ManagedRuntime.make(makeServerLayer(input));

  const run = <A>(name: string, effect: Effect.Effect<A, unknown, ServerRuntimeEnv>) =>
    runtime
      .runPromise(effect.pipe(Effect.withSpan(`context-map.opencode-plugin.${name}`)))
      .catch((error) => {
        throw toThrowable(error);
      });

  const root = getWorkspaceRoot(input.worktree, input.directory);
  await run(
    "startup.log",
    Effect.gen(function* () {
      const logger = yield* PluginLogger;
      yield* logger.log({
        level: "info",
        message: "Plugin initialized",
        extra: { directory: input.directory, worktree: input.worktree, root },
      }).pipe(Effect.ignore);
    }),
  );

  return {
    event: async (eventInput) => {
      await run("event", onEvent(eventInput));
    },
    "experimental.chat.system.transform": async (hookInput, output) => {
      await run(
        "experimental.chat.system.transform",
        injectAtlasIntoSystem({ input: hookInput, output }),
      );
    },
  } satisfies Hooks;
};

export default {
  id: PLUGIN_ID,
  server: CmapOpenCodePlugin,
} satisfies PluginModule;
