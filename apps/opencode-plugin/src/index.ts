import type { Plugin, PluginInput, PluginModule } from "@opencode-ai/plugin";

type ShellTextRunner = PluginInput["$"];
type OpencodeClient = PluginInput["client"];

const SERVICE_NAME = "context-map-opencode-plugin";
const BUILD_DEBOUNCE_MS = 2_000;

const textFileChangedEvents = new Set(["file.edited", "file.watcher.updated"]);

type LogLevel = "debug" | "info" | "warn" | "error";

const formatError = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

const log = async (
  client: OpencodeClient,
  level: LogLevel,
  message: string,
  extra?: Record<string, unknown>,
): Promise<void> => {
  try {
    await client.app.log({
      body: {
        service: SERVICE_NAME,
        level,
        message,
        extra,
      },
    });
  } catch {
    // Logging must never affect hook behavior.
  }
};

const readText = async (
  $: ShellTextRunner,
  path: string,
): Promise<string | undefined> => {
  const exists = (await $`test -f ${path} && echo "yes"`.text()).trim() === "yes";
  if (!exists) return undefined;

  const content = await $`cat ${path}`.text();
  return content.trim().length > 0 ? content : undefined;
};

const hasCmapDirectory = async ($: ShellTextRunner, root: string): Promise<boolean> =>
  (await $`test -d ${root}/.cmap && echo "yes"`.text()).trim() === "yes";

export const getWorkspaceRoot = (worktree: string, directory: string): string =>
  worktree || directory;

export const getSessionId = (event: { properties?: unknown }): string | undefined => {
  const properties = event.properties;
  if (typeof properties !== "object" || properties === null) return undefined;

  const sessionID = (properties as { sessionID?: unknown }).sessionID;
  return typeof sessionID === "string" && sessionID.length > 0 ? sessionID : undefined;
};

export const shouldRunChangedOnlyBuild = (eventType: string): boolean =>
  textFileChangedEvents.has(eventType);

const injectAtlas = async (
  client: OpencodeClient,
  $: ShellTextRunner,
  root: string,
  sessionID: string,
): Promise<void> => {
  const atlasPath = `${root}/.cmap/views/ROOT_ATLAS.md`;
  const content = await readText($, atlasPath);
  if (!content) return;

  await client.session.prompt({
    path: { id: sessionID },
    body: {
      noReply: true,
      parts: [
        {
          type: "text",
          text: `[System] Knowledge Base Atlas for this repository:\n\n${content}`,
          synthetic: true,
        },
      ],
    },
  });
};

const runChangedOnlyBuild = async (
  $: ShellTextRunner,
  root: string,
): Promise<void> => {
  const initialized = await hasCmapDirectory($, root);
  if (!initialized) return;

  await $`cmap build --root ${root} --changed-only`.text();
};

export const CmapOpenCodePlugin: Plugin = async ({ $, client, directory, worktree }) => {
  const root = getWorkspaceRoot(worktree, directory);
  let buildTimeout: ReturnType<typeof setTimeout> | undefined;

  await log(client, "info", "Plugin initialized", { directory, worktree, root });

  return {
    event: async ({ event }) => {
      if (event.type === "session.created") {
        const sessionID = getSessionId(event);
        if (!sessionID) return;

        try {
          await injectAtlas(client, $, root, sessionID);
        } catch (error) {
          await log(client, "error", "Failed to inject ROOT_ATLAS.md", {
            root,
            sessionID,
            error: formatError(error),
          });
        }
        return;
      }

      if (!shouldRunChangedOnlyBuild(event.type)) return;

      if (buildTimeout) clearTimeout(buildTimeout);
      buildTimeout = setTimeout(() => {
        void (async () => {
          try {
            await runChangedOnlyBuild($, root);
            await log(client, "debug", "Changed-only cmap build completed", { root });
          } catch (error) {
            await log(client, "error", "Changed-only cmap build failed", {
              root,
              error: formatError(error),
            });
          }
        })();
      }, BUILD_DEBOUNCE_MS);
    },
  };
};

export default {
  id: "context-map-opencode-plugin",
  server: CmapOpenCodePlugin,
} satisfies PluginModule;
