import { Schema } from "effect";

export class OpencodeClientError extends Schema.TaggedError<OpencodeClientError>()(
  "OpencodeClientError",
  {
    operation: Schema.String,
    message: Schema.String,
    cause: Schema.Unknown,
  },
) {}

export class ShellCommandError extends Schema.TaggedError<ShellCommandError>()(
  "ShellCommandError",
  {
    command: Schema.String,
    message: Schema.String,
    cause: Schema.Unknown,
  },
) {}

export const toThrowable = (error: unknown): Error =>
  error instanceof Error ? error : new Error(String(error));

export const formatError = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);
