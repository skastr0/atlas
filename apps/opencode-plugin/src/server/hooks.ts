import type { Hooks } from "@opencode-ai/plugin";
import { Effect, Fiber } from "effect";
import { BUILD_DEBOUNCE_MS } from "../shared/constants";
import { formatError, ShellCommandError } from "../shared/errors";
import { PluginLogger } from "../shared/logger";
import { AtlasPluginContext } from "./layers";

type EventInput = Parameters<NonNullable<Hooks["event"]>>[0];

const atlasInitializationEvents = new Set(["server.connected", "session.created"]);
const textFileChangedEvents = new Set(["file.edited", "file.watcher.updated"]);

let changedOnlyBuildFiber: Fiber.RuntimeFiber<void, never> | undefined;
let changedOnlyBuildInFlight = false;
let changedOnlyBuildPending = false;

export const getSessionId = (event: { properties?: unknown }): string | undefined => {
  const properties = event.properties;
  if (typeof properties !== "object" || properties === null) return undefined;

  const sessionID = (properties as { sessionID?: unknown }).sessionID;
  return typeof sessionID === "string" && sessionID.length > 0 ? sessionID : undefined;
};

export const shouldRunChangedOnlyBuild = (eventType: string): boolean =>
  textFileChangedEvents.has(eventType);

export const shouldRunAtlasInit = (eventType: string): boolean =>
  atlasInitializationEvents.has(eventType);

const hasAtlasCli = Effect.fn("Atlas.hasCli")(function* () {
  const context = yield* AtlasPluginContext;

  const atlasPath = yield* Effect.tryPromise({
    try: () => context.$`command -v atlas`.text(),
    catch: (cause) =>
      new ShellCommandError({
        command: "command -v atlas",
        message: "atlas CLI is not available",
        cause,
      }),
  }).pipe(Effect.catchAll(() => Effect.succeed("")));

  return atlasPath.trim().length > 0;
});

const runAtlasInit = Effect.fn("Atlas.runInit")(function* (reason: string) {
  const context = yield* AtlasPluginContext;
  const logger = yield* PluginLogger;

  if (!(yield* hasAtlasCli())) return;

  yield* Effect.tryPromise({
    try: () => context.$`atlas init --root ${context.root} --quiet`.text(),
    catch: (cause) =>
      new ShellCommandError({
        command: "atlas init",
        message: "atlas init failed",
        cause,
      }),
  });

  yield* logger.log({
    level: "info",
    message: "atlas init completed",
    extra: { root: context.root, reason },
  }).pipe(Effect.ignore);
});

const runChangedOnlyBuild = Effect.fn("Atlas.runChangedOnlyBuild")(function* () {
  const context = yield* AtlasPluginContext;
  const logger = yield* PluginLogger;

  if (!(yield* hasAtlasCli())) return;

  const initialized = yield* Effect.tryPromise({
    try: async () =>
      (await context.$`test -d ${context.root}/.atlas && echo "yes"`.text()).trim() === "yes",
    catch: (cause) =>
      new ShellCommandError({
        command: "test .atlas",
        message: "Atlas is not initialized",
        cause,
      }),
  }).pipe(Effect.catchAll(() => Effect.succeed(false)));
  if (!initialized) return;

  yield* Effect.tryPromise({
    try: () => context.$`atlas build --root ${context.root} --changed-only`.text(),
    catch: (cause) =>
      new ShellCommandError({
        command: "atlas build --changed-only",
        message: "Changed-only atlas build failed",
        cause,
      }),
  });

  yield* logger.log({
    level: "debug",
    message: "Changed-only atlas build completed",
    extra: { root: context.root },
  }).pipe(Effect.ignore);
});

const logChangedOnlyBuildError = (error: unknown) =>
  Effect.gen(function* () {
    const context = yield* AtlasPluginContext;
    const logger = yield* PluginLogger;

    yield* logger.log({
      level: "error",
      message: "Changed-only atlas build failed",
      extra: { root: context.root, error: formatError(error) },
    }).pipe(Effect.ignore);
  });

const runQueuedChangedOnlyBuild = Effect.fn("Atlas.runQueuedChangedOnlyBuild")(function* () {
  if (changedOnlyBuildInFlight) {
    changedOnlyBuildPending = true;
    return;
  }

  changedOnlyBuildInFlight = true;
  yield* Effect.gen(function* () {
    do {
      changedOnlyBuildPending = false;
      yield* runChangedOnlyBuild().pipe(Effect.catchAll(logChangedOnlyBuildError));
    } while (changedOnlyBuildPending);
  }).pipe(
    Effect.ensuring(
      Effect.sync(() => {
        changedOnlyBuildInFlight = false;
      }),
    ),
  );
});

const scheduleChangedOnlyBuild = Effect.fn("Atlas.scheduleChangedOnlyBuild")(function* () {
  if (changedOnlyBuildFiber) {
    yield* Fiber.interrupt(changedOnlyBuildFiber).pipe(Effect.ignore);
  }

  changedOnlyBuildFiber = yield* Effect.sleep(BUILD_DEBOUNCE_MS).pipe(
    Effect.andThen(runQueuedChangedOnlyBuild()),
    Effect.forkDaemon,
  );
});

export const onEvent = Effect.fn("ServerHooks.onEvent")(function* (input: EventInput) {
  const eventType = input.event.type;
  const logger = yield* PluginLogger;

  if (shouldRunAtlasInit(eventType)) {
    yield* runAtlasInit(eventType).pipe(
      Effect.catchAll((error) =>
        logger.log({
          level: "error",
          message: "atlas init failed",
          extra: { reason: eventType, error: formatError(error) },
        }).pipe(Effect.ignore),
      ),
    );
  }

  if (!shouldRunChangedOnlyBuild(eventType)) return;

  yield* scheduleChangedOnlyBuild();
});
