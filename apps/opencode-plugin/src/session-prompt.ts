type SessionPromptContext = {
  readonly agent: string;
  readonly model: { readonly providerID: string; readonly modelID: string };
  readonly variant?: string;
};

export type SessionPromptClient = {
  readonly session: {
    readonly messages: (options: any) => Promise<unknown>;
    readonly prompt: (options: any) => Promise<unknown>;
  };
};

type PromptPart = {
  readonly type: "text";
  readonly text: string;
  readonly synthetic?: boolean;
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function getString(record: Record<string, unknown>, key: string): string | undefined {
  const value = Reflect.get(record, key);
  return typeof value === "string" ? value : undefined;
}

function extractMessages(response: unknown): ReadonlyArray<unknown> {
  if (Array.isArray(response)) return response;
  if (!isRecord(response)) return [];

  const data = Reflect.get(response, "data");
  return Array.isArray(data) ? data : [];
}

function extractMessageInfo(message: unknown): Record<string, unknown> | undefined {
  if (!isRecord(message)) return undefined;

  const info = Reflect.get(message, "info");
  return isRecord(info) ? info : message;
}

function extractPromptContext(messages: ReadonlyArray<unknown>): SessionPromptContext | undefined {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const info = extractMessageInfo(messages[index]);
    if (!info || Reflect.get(info, "role") !== "user") continue;

    const agent = getString(info, "agent");
    const model = Reflect.get(info, "model");
    if (!agent || !isRecord(model)) continue;

    const providerID = getString(model, "providerID");
    const modelID = getString(model, "modelID");
    if (!providerID || !modelID) continue;

    const variant = getString(model, "variant");
    return {
      agent,
      model: { providerID, modelID },
      ...(variant === undefined ? {} : { variant }),
    };
  }

  return undefined;
}

export async function readSessionPromptContext(
  client: SessionPromptClient,
  sessionID: string,
): Promise<SessionPromptContext | undefined> {
  const response = await client.session.messages({ path: { id: sessionID } });
  return extractPromptContext(extractMessages(response));
}

export async function promptWithSessionContext(
  client: SessionPromptClient,
  sessionID: string,
  input: {
    readonly noReply: boolean;
    readonly parts: ReadonlyArray<PromptPart>;
  },
): Promise<void> {
  const context = await readSessionPromptContext(client, sessionID);
  if (!context) {
    throw new Error(
      `Refusing to inject prompt into ${sessionID}: no existing user message with agent/model context`,
    );
  }

  await client.session.prompt({
    path: { id: sessionID },
    body: {
      agent: context.agent,
      model: context.model,
      ...(context.variant === undefined ? {} : { variant: context.variant }),
      noReply: input.noReply,
      parts: [...input.parts],
    },
  });
}
