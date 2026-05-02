import type { Hooks } from "@opencode-ai/plugin";
import { Effect } from "effect";
import { BUILD_DEBOUNCE_MS } from "../shared/constants";
import { formatError, ShellCommandError } from "../shared/errors";
import { PluginLogger } from "../shared/logger";
import { buildAtlasSystemContext, readAtlas } from "./atlas";
import { CmapPluginContext } from "./layers";

type EventInput = Parameters<NonNullable<Hooks["event"]>>[0];
type SystemTransformInput = Parameters<
  NonNullable<Hooks["experimental.chat.system.transform"]>
>[0];
type SystemTransformOutput = Parameters<
  NonNullable<Hooks["experimental.chat.system.transform"]>
>[1];

const cmapInitializationEvents = new Set(["server.connected", "session.created"]);
const textFileChangedEvents = new Set(["file.edited", "file.watcher.updated"]);
const atlasRefreshEvents = new Set(["session.created", "session.compacted"]);

let changedOnlyBuildTimeout: ReturnType<typeof setTimeout> | undefined;

export const getSessionId = (event: { properties?: unknown }): string | undefined => {
  const properties = event.properties;
  if (typeof properties !== "object" || properties === null) return undefined;

  const sessionID = (properties as { sessionID?: unknown }).sessionID;
  return typeof sessionID === "string" && sessionID.length > 0 ? sessionID : undefined;
};

export const shouldRunChangedOnlyBuild = (eventType: string): boolean =>
  textFileChangedEvents.has(eventType);

export const shouldRunCmapInit = (eventType: string): boolean =>
  cmapInitializationEvents.has(eventType);

export const shouldRefreshAtlasContext = (eventType: string): boolean =>
  atlasRefreshEvents.has(eventType);

const runCmapInit = Effect.fn("Cmap.runInit")(function* (reason: string) {
  const context = yield* CmapPluginContext;
  const logger = yield* PluginLogger;

  yield* Effect.tryPromise({
    try: () => context.$`cmap init --root ${context.root} --quiet`.text(),
    catch: (cause) =>
      new ShellCommandError({
        command: "cmap init",
        message: "cmap init failed",
        cause,
      }),
  });

  yield* logger.log({
    level: "info",
    message: "cmap init completed",
    extra: { root: context.root, reason },
  }).pipe(Effect.ignore);
});

const runChangedOnlyBuild = Effect.fn("Cmap.runChangedOnlyBuild")(function* () {
  const context = yield* CmapPluginContext;
  const logger = yield* PluginLogger;

  const initialized = yield* Effect.tryPromise({
    try: async () =>
      (await context.$`test -d ${context.root}/.cmap && echo "yes"`.text()).trim() === "yes",
    catch: () => false,
  });
  if (!initialized) return;

  yield* Effect.tryPromise({
    try: () => context.$`cmap build --root ${context.root} --changed-only`.text(),
    catch: (cause) =>
      new ShellCommandError({
        command: "cmap build --changed-only",
        message: "Changed-only cmap build failed",
        cause,
      }),
  });

  yield* logger.log({
    level: "debug",
    message: "Changed-only cmap build completed",
    extra: { root: context.root },
  }).pipe(Effect.ignore);
});

export const injectAtlasIntoSystem = Effect.fn("ServerHooks.injectAtlasIntoSystem")(
  function* ({
    input,
    output,
  }: {
    readonly input: SystemTransformInput;
    readonly output: SystemTransformOutput;
  }) {
    const atlas = yield* readAtlas();
    if (!atlas) return;

    output.system.push(buildAtlasSystemContext(atlas));

    const logger = yield* PluginLogger;
    yield* logger.log({
      level: "debug",
      message: "Injected ROOT_ATLAS.md into system prompt",
      extra: { sessionID: input.sessionID },
    }).pipe(Effect.ignore);
  },
);

export const onEvent = Effect.fn("ServerHooks.onEvent")(function* (input: EventInput) {
  const eventType = input.event.type;
  const logger = yield* PluginLogger;

  if (shouldRunCmapInit(eventType)) {
    yield* runCmapInit(eventType).pipe(
      Effect.catchAll((error) =>
        logger.log({
          level: "error",
          message: "cmap init failed",
          extra: { reason: eventType, error: formatError(error) },
        }).pipe(Effect.ignore),
      ),
    );
  }

  if (shouldRefreshAtlasContext(eventType)) {
    yield* logger.log({
      level: "debug",
      message: "Atlas context will be refreshed on next model request",
      extra: { eventType, sessionID: getSessionId(input.event) },
    }).pipe(Effect.ignore);
  }

  if (!shouldRunChangedOnlyBuild(eventType)) return;

  const context = yield* CmapPluginContext;
  yield* Effect.sync(() => {
    if (changedOnlyBuildTimeout) clearTimeout(changedOnlyBuildTimeout);
    changedOnlyBuildTimeout = setTimeout(() => {
      void Effect.runPromise(
        runChangedOnlyBuild().pipe(
          Effect.provideService(CmapPluginContext, context),
          Effect.provideService(PluginLogger, logger),
          Effect.catchAll((error) =>
            logger.log({
              level: "error",
              message: "Changed-only cmap build failed",
              extra: { root: context.root, error: formatError(error) },
            }).pipe(Effect.ignore),
          ),
        ),
      );
    }, BUILD_DEBOUNCE_MS);
  });
});
