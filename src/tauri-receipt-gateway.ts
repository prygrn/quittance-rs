import type { TemplateInfo } from "./api";
import type { InvokeCommand } from "./invoke-command";
import type { ReceiptGateway } from "./receipt-gateway";
import type { PreviewRequest } from "./screen-state";
import { toLocalIsoDate } from "./to-local-iso-date";

/**
 * Passerelle vers les commandes Tauri `list_templates`, `preview_receipt` et
 * `send_receipt`. `now` fournit l'instant d'appel, dont la date locale devient l'`issueDate`.
 */
export function createTauriReceiptGateway(dependencies: {
  readonly invokeCommand: InvokeCommand;
  readonly now: () => Date;
}): ReceiptGateway {
  const { invokeCommand, now } = dependencies;
  // La date d'émission (« Fait à …, le … ») est celle du jour de l'appel, en heure locale.
  const receiptArguments = (request: PreviewRequest): Readonly<Record<string, unknown>> => ({
    templateId: request.templateId,
    input: request.input,
    issueDate: toLocalIsoDate(now()),
  });

  return {
    listTemplates: async () => {
      const response = await invokeCommand("list_templates", {});
      if (!isTemplateInfoList(response)) {
        throw invalidResponseError({
          command: "list_templates",
          expected: "TemplateInfo[]",
          response,
        });
      }
      return response;
    },
    renderPreview: async (request: PreviewRequest) => {
      const response = await invokeCommand("preview_receipt", receiptArguments(request));
      if (typeof response !== "string") {
        throw invalidResponseError({ command: "preview_receipt", expected: "string", response });
      }
      return response;
    },
    sendReceipt: async (request: PreviewRequest) => {
      const response = await invokeCommand("send_receipt", receiptArguments(request));
      if (response !== null) {
        throw invalidResponseError({ command: "send_receipt", expected: "null", response });
      }
    },
  };
}

function isTemplateInfoList(value: unknown): value is readonly TemplateInfo[] {
  return Array.isArray(value) && value.every(isTemplateInfo);
}

function isTemplateInfo(value: unknown): value is TemplateInfo {
  return hasOwnString(value, "id") && hasOwnString(value, "label");
}

function hasOwnString(value: unknown, field: string): boolean {
  return (
    typeof value === "object" &&
    value !== null &&
    Object.hasOwn(value, field) &&
    typeof (value as Readonly<Record<string, unknown>>)[field] === "string"
  );
}

// Sans `code`, l'erreur est traitée comme `unknown` par l'écran, qui la journalise.
function invalidResponseError(details: {
  readonly command: string;
  readonly expected: string;
  readonly response: unknown;
}): Error {
  return new Error(
    `[UI_INVALID_COMMAND_RESPONSE] command ${details.command} returned ${describeValue(details.response)} instead of ${details.expected}`,
  );
}

// Type seul, sans le contenu : une réponse peut être volumineuse ou contenir des données personnelles.
function describeValue(value: unknown): string {
  if (value === null) {
    return "null";
  }
  return Array.isArray(value) ? "an array" : `a value of type ${typeof value}`;
}
