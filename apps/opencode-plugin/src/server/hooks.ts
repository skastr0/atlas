import type { Hooks } from "@opencode-ai/plugin";
import { Effect, Fiber } from "effect";
import { BUILD_DEBOUNCE_MS } from "../shared/constants";
import { formatError, ShellCommandError } from "../shared/errors";
import { PluginLogger } from "../shared/logger";
import { CmapPluginContext } from "./layers";

type EventInput = Parameters<NonNullable<Hooks["event"]>>[0];

const cmapInitializationEvents = new Set(["server.connected", "session.created"]);
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

export const shouldRunCmapInit = (eventType: string): boolean =>
  cmapInitializationEvents.has(eventType);

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

const logChangedOnlyBuildError = (error: unknown) =>
  Effect.gen(function* () {
    const context = yield* CmapPluginContext;
    const logger = yield* PluginLogger;

    yield* logger.log({
      level: "error",
      message: "Changed-only cmap build failed",
      extra: { root: context.root, error: formatError(error) },
    }).pipe(Effect.ignore);
  });

const runQueuedChangedOnlyBuild = Effect.fn("Cmap.runQueuedChangedOnlyBuild")(function* () {
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

const scheduleChangedOnlyBuild = Effect.fn("Cmap.scheduleChangedOnlyBuild")(function* () {
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

  if (!shouldRunChangedOnlyBuild(eventType)) return;

  yield* scheduleChangedOnlyBuild();
});
