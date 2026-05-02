import { Effect } from "effect";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { ROOT_ATLAS_RELATIVE_PATH } from "../shared/constants";
import { CmapPluginContext } from "./layers";

export const buildAtlasSystemContext = (content: string): string =>
  `[System] Knowledge Base Atlas for this repository:\n\n${content}`;

export const buildAtlasPromptPart = (content: string) => ({
  type: "text" as const,
  text: buildAtlasSystemContext(content),
  synthetic: true,
});

export const readAtlas = Effect.fn("Atlas.read")(function* () {
  const context = yield* CmapPluginContext;
  const atlasPath = join(context.root, ROOT_ATLAS_RELATIVE_PATH);

  const content = yield* Effect.tryPromise({
    try: () => readFile(atlasPath, "utf8"),
    catch: (cause) => cause,
  }).pipe(Effect.catchAll(() => Effect.succeed(undefined)));

  if (content === undefined) return undefined;

  const trimmed = content.trim();
  return trimmed.length > 0 ? content : undefined;
});
