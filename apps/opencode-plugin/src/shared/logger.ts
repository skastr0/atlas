import { Context, Effect, Layer } from "effect";
import { PLUGIN_SERVICE } from "./constants";
import { OpencodeClientError } from "./errors";

export type LogLevel = "debug" | "info" | "warn" | "error";

export type LogEntry = {
  readonly level: LogLevel;
  readonly message: string;
  readonly extra?: unknown;
};

export class PluginLogger extends Context.Tag("@context-map/opencode-plugin/PluginLogger")<
  PluginLogger,
  {
    readonly log: (entry: LogEntry) => Effect.Effect<void, OpencodeClientError>;
  }
>() {}

export const makeServerLoggerLayer = (client: any) =>
  Layer.succeed(PluginLogger, {
    log: (entry) =>
      Effect.tryPromise({
        try: async () => {
          await Promise.resolve(
            client.app?.log?.({
              body: {
                service: PLUGIN_SERVICE,
                level: entry.level,
                message: entry.message,
                extra: entry.extra,
              },
            }),
          );
        },
        catch: (cause) =>
          new OpencodeClientError({
            operation: "client.app.log",
            message: "Failed to write to the opencode app log",
            cause,
          }),
      }),
  });
