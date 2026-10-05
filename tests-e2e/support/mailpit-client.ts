import { JSON_READER } from "./json-reader";
import { MAILPIT } from "./mailpit";

// API HTTP v1 du Mailpit local, celui du service CI ou du conteneur lancé en local.
const MAILPIT_API_URL = `http://${MAILPIT.host}:${MAILPIT.apiPort}/api/v1`;

/** Message reçu par Mailpit, réduit aux données vérifiées par les scénarios. */
export interface MailpitMessage {
  readonly id: string;
  readonly fromAddress: string;
  readonly toAddresses: readonly string[];
  readonly bccAddresses: readonly string[];
  readonly subject: string;
  readonly attachments: readonly MailpitAttachment[];
}

/** Pièce jointe d'un message, telle que Mailpit la décrit. */
export interface MailpitAttachment {
  readonly partId: string;
  readonly fileName: string;
  readonly contentType: string;
  readonly size: number;
}

async function request(options: {
  readonly path: string;
  readonly method: "GET" | "DELETE";
}): Promise<Response> {
  const url = `${MAILPIT_API_URL}${options.path}`;
  let response: Response;
  try {
    response = await fetch(url, { method: options.method });
  } catch (err: unknown) {
    throw new Error(
      `[E2E_MAILPIT_UNREACHABLE] ${options.method} ${url} failed: is Mailpit running on ${MAILPIT.host} (SMTP ${MAILPIT.smtpPort}, API ${MAILPIT.apiPort})?`,
      { cause: err },
    );
  }
  if (!response.ok) {
    throw new Error(
      `[E2E_MAILPIT_REQUEST_FAILED] ${options.method} ${url} answered ${response.status}: ${await response.text()}`,
    );
  }
  return response;
}

async function requestJson(path: string): Promise<unknown> {
  const response = await request({ path, method: "GET" });
  return response.json();
}

function readAddresses(options: {
  readonly message: Readonly<Record<string, unknown>>;
  readonly name: string;
  readonly context: string;
}): readonly string[] {
  return JSON_READER.requireArray({ record: options.message, ...options }).map((entry, index) =>
    JSON_READER.requireString({
      record: JSON_READER.requireRecord({
        value: entry,
        context: `${options.context}.${options.name}[${index}]`,
      }),
      name: "Address",
      context: `${options.context}.${options.name}[${index}]`,
    }),
  );
}

function readAttachment(options: {
  readonly value: unknown;
  readonly context: string;
}): MailpitAttachment {
  const record = JSON_READER.requireRecord(options);
  const field = (name: string): { record: typeof record; name: string; context: string } => ({
    record,
    name,
    context: options.context,
  });
  return {
    partId: JSON_READER.requireString(field("PartID")),
    fileName: JSON_READER.requireString(field("FileName")),
    contentType: JSON_READER.requireString(field("ContentType")),
    size: JSON_READER.requireNumber(field("Size")),
  };
}

async function listMessageIds(): Promise<readonly string[]> {
  const context = "GET /messages";
  const summary = JSON_READER.requireRecord({ value: await requestJson("/messages"), context });
  return JSON_READER.requireArray({ record: summary, name: "messages", context }).map(
    (entry, index) => {
      const entryContext = `${context}.messages[${index}]`;
      return JSON_READER.requireString({
        record: JSON_READER.requireRecord({ value: entry, context: entryContext }),
        name: "ID",
        context: entryContext,
      });
    },
  );
}

async function getMessage(id: string): Promise<MailpitMessage> {
  const context = `GET /message/${id}`;
  const message = JSON_READER.requireRecord({
    value: await requestJson(`/message/${encodeURIComponent(id)}`),
    context,
  });
  const sender = JSON_READER.requireRecord({ value: message["From"], context: `${context}.From` });
  return {
    id,
    fromAddress: JSON_READER.requireString({
      record: sender,
      name: "Address",
      context: `${context}.From`,
    }),
    toAddresses: readAddresses({ message, name: "To", context }),
    bccAddresses: readAddresses({ message, name: "Bcc", context }),
    subject: JSON_READER.requireString({ record: message, name: "Subject", context }),
    attachments: JSON_READER.requireArray({ record: message, name: "Attachments", context }).map(
      (value, index) => readAttachment({ value, context: `${context}.Attachments[${index}]` }),
    ),
  };
}

/**
 * Accès au Mailpit local : vider la boîte avant un scénario, puis relire ce que l'app a
 * réellement envoyé.
 */
export const MAILPIT_CLIENT = {
  /** Supprime tous les messages : sans liste d'identifiants, Mailpit vide la boîte. */
  deleteAllMessages: async (): Promise<void> => {
    await request({ path: "/messages", method: "DELETE" });
  },
  listMessageIds,
  getMessage,
  /** Contenu brut d'une pièce jointe. */
  getAttachmentContent: async (options: {
    readonly messageId: string;
    readonly partId: string;
  }): Promise<Uint8Array> => {
    const response = await request({
      path: `/message/${encodeURIComponent(options.messageId)}/part/${encodeURIComponent(options.partId)}`,
      method: "GET",
    });
    return new Uint8Array(await response.arrayBuffer());
  },
} as const;
