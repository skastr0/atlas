// @bun
// src/index.ts
import { Effect as Effect3, ManagedRuntime } from "effect";

// src/server/hooks.ts
import { Effect as Effect2, Fiber } from "effect";

// src/shared/constants.ts
var PLUGIN_ID = "atlas-opencode-plugin";
var PLUGIN_SERVICE = "atlas-opencode-plugin";
var BUILD_DEBOUNCE_MS = 2000;

// src/shared/errors.ts
import { Schema } from "effect";

class OpencodeClientError extends Schema.TaggedError()("OpencodeClientError", {
  operation: Schema.String,
  message: Schema.String,
  cause: Schema.Unknown
}) {
}

class ShellCommandError extends Schema.TaggedError()("ShellCommandError", {
  command: Schema.String,
  message: Schema.String,
  cause: Schema.Unknown
}) {
}
var toThrowable = (error) => error instanceof Error ? error : new Error(String(error));
var formatError = (error) => error instanceof Error ? error.message : String(error);

// src/shared/logger.ts
import { Context, Effect, Layer } from "effect";
class PluginLogger extends Context.Tag("@skastr0/atlas-opencode-plugin/PluginLogger")() {
}
var makeServerLoggerLayer = (client) => Layer.succeed(PluginLogger, {
  log: (entry) => Effect.tryPromise({
    try: async () => {
      await Promise.resolve(client.app?.log?.({
        body: {
          service: PLUGIN_SERVICE,
          level: entry.level,
          message: entry.message,
          extra: entry.extra
        }
      }));
    },
    catch: (cause) => new OpencodeClientError({
      operation: "client.app.log",
      message: "Failed to write to the opencode app log",
      cause
    })
  })
});

// src/server/layers.ts
import { Context as Context2, Layer as Layer2 } from "effect";
class AtlasPluginContext extends Context2.Tag("@skastr0/atlas-opencode-plugin/AtlasPluginContext")() {
}
var getWorkspaceRoot = (worktree, directory) => worktree || directory;
var makeServerLayer = (input) => {
  const root = getWorkspaceRoot(input.worktree, input.directory);
  return Layer2.mergeAll(Layer2.succeed(AtlasPluginContext, {
    client: input.client,
    $: input.$,
    directory: input.directory,
    worktree: input.worktree,
    root
  }), makeServerLoggerLayer(input.client));
};

// src/server/hooks.ts
var atlasInitializationEvents = new Set(["server.connected", "session.created"]);
var textFileChangedEvents = new Set(["file.edited", "file.watcher.updated"]);
var changedOnlyBuildFiber;
var changedOnlyBuildInFlight = false;
var changedOnlyBuildPending = false;
var getSessionId = (event) => {
  const properties = event.properties;
  if (typeof properties !== "object" || properties === null)
    return;
  const sessionID = properties.sessionID;
  return typeof sessionID === "string" && sessionID.length > 0 ? sessionID : undefined;
};
var shouldRunChangedOnlyBuild = (eventType) => textFileChangedEvents.has(eventType);
var shouldRunAtlasInit = (eventType) => atlasInitializationEvents.has(eventType);
var runAtlasInit = Effect2.fn("Atlas.runInit")(function* (reason) {
  const context = yield* AtlasPluginContext;
  const logger = yield* PluginLogger;
  yield* Effect2.tryPromise({
    try: () => context.$`atlas init --root ${context.root} --quiet`.text(),
    catch: (cause) => new ShellCommandError({
      command: "atlas init",
      message: "atlas init failed",
      cause
    })
  });
  yield* logger.log({
    level: "info",
    message: "atlas init completed",
    extra: { root: context.root, reason }
  }).pipe(Effect2.ignore);
});
var runChangedOnlyBuild = Effect2.fn("Atlas.runChangedOnlyBuild")(function* () {
  const context = yield* AtlasPluginContext;
  const logger = yield* PluginLogger;
  const initialized = yield* Effect2.tryPromise({
    try: async () => (await context.$`test -d ${context.root}/.atlas && echo "yes"`.text()).trim() === "yes",
    catch: () => false
  });
  if (!initialized)
    return;
  yield* Effect2.tryPromise({
    try: () => context.$`atlas build --root ${context.root} --changed-only`.text(),
    catch: (cause) => new ShellCommandError({
      command: "atlas build --changed-only",
      message: "Changed-only atlas build failed",
      cause
    })
  });
  yield* logger.log({
    level: "debug",
    message: "Changed-only atlas build completed",
    extra: { root: context.root }
  }).pipe(Effect2.ignore);
});
var logChangedOnlyBuildError = (error) => Effect2.gen(function* () {
  const context = yield* AtlasPluginContext;
  const logger = yield* PluginLogger;
  yield* logger.log({
    level: "error",
    message: "Changed-only atlas build failed",
    extra: { root: context.root, error: formatError(error) }
  }).pipe(Effect2.ignore);
});
var runQueuedChangedOnlyBuild = Effect2.fn("Atlas.runQueuedChangedOnlyBuild")(function* () {
  if (changedOnlyBuildInFlight) {
    changedOnlyBuildPending = true;
    return;
  }
  changedOnlyBuildInFlight = true;
  yield* Effect2.gen(function* () {
    do {
      changedOnlyBuildPending = false;
      yield* runChangedOnlyBuild().pipe(Effect2.catchAll(logChangedOnlyBuildError));
    } while (changedOnlyBuildPending);
  }).pipe(Effect2.ensuring(Effect2.sync(() => {
    changedOnlyBuildInFlight = false;
  })));
});
var scheduleChangedOnlyBuild = Effect2.fn("Atlas.scheduleChangedOnlyBuild")(function* () {
  if (changedOnlyBuildFiber) {
    yield* Fiber.interrupt(changedOnlyBuildFiber).pipe(Effect2.ignore);
  }
  changedOnlyBuildFiber = yield* Effect2.sleep(BUILD_DEBOUNCE_MS).pipe(Effect2.andThen(runQueuedChangedOnlyBuild()), Effect2.forkDaemon);
});
var onEvent = Effect2.fn("ServerHooks.onEvent")(function* (input) {
  const eventType = input.event.type;
  const logger = yield* PluginLogger;
  if (shouldRunAtlasInit(eventType)) {
    yield* runAtlasInit(eventType).pipe(Effect2.catchAll((error) => logger.log({
      level: "error",
      message: "atlas init failed",
      extra: { reason: eventType, error: formatError(error) }
    }).pipe(Effect2.ignore)));
  }
  if (!shouldRunChangedOnlyBuild(eventType))
    return;
  yield* scheduleChangedOnlyBuild();
});

// src/index.ts
var AtlasOpenCodePlugin = async (input) => {
  const runtime = ManagedRuntime.make(makeServerLayer(input));
  const run = (name, effect) => runtime.runPromise(effect.pipe(Effect3.withSpan(`atlas.opencode-plugin.${name}`))).catch((error) => {
    throw toThrowable(error);
  });
  const root = getWorkspaceRoot(input.worktree, input.directory);
  await run("startup.log", Effect3.gen(function* () {
    const logger = yield* PluginLogger;
    yield* logger.log({
      level: "info",
      message: "Plugin initialized",
      extra: { directory: input.directory, worktree: input.worktree, root }
    }).pipe(Effect3.ignore);
  }));
  return {
    event: async (eventInput) => {
      await run("event", onEvent(eventInput));
    }
  };
};
var src_default = {
  id: PLUGIN_ID,
  server: AtlasOpenCodePlugin
};
export {
  shouldRunChangedOnlyBuild,
  shouldRunAtlasInit,
  getWorkspaceRoot,
  getSessionId,
  src_default as default,
  AtlasOpenCodePlugin
};
