import type { ReceiptInput, TemplateInfo } from "./api";
import { invalidCommandResponseError } from "./invalid-command-response-error";
import type { InvokeCommand } from "./invoke-command";
import type { ReceiptGateway } from "./receipt-gateway";
import { readOwnField } from "./read-own-field";
import type { PreviewRequest } from "./screen-state";
import { toLocalIsoDate } from "./to-local-iso-date";

/** Arguments communs de `preview_receipt` et `send_receipt`, en camelCase côté UI. */
type ReceiptCommandArguments = {
  readonly templateId: string;
  readonly input: ReceiptInput;
  /** Date locale du jour au format `YYYY-MM-DD`, reprise dans « Fait à …, le … ». */
  readonly issueDate: string;
};

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
  const receiptArguments = (request: PreviewRequest): ReceiptCommandArguments => ({
    templateId: request.templateId,
    input: request.input,
    issueDate: toLocalIsoDate(now()),
  });

  return {
    listTemplates: async (): Promise<readonly TemplateInfo[]> => {
      const response = await invokeCommand("list_templates", {});
      if (!isTemplateInfoList(response)) {
        throw invalidCommandResponseError({
          command: "list_templates",
          expected: "TemplateInfo[]",
          response,
        });
      }
      return response;
    },
    renderPreview: async (request: PreviewRequest): Promise<string> => {
      const response = await invokeCommand("preview_receipt", receiptArguments(request));
      if (typeof response !== "string") {
        throw invalidCommandResponseError({
          command: "preview_receipt",
          expected: "string",
          response,
        });
      }
      return response;
    },
    sendReceipt: async (request: PreviewRequest): Promise<void> => {
      const response = await invokeCommand("send_receipt", receiptArguments(request));
      if (response !== null) {
        throw invalidCommandResponseError({ command: "send_receipt", expected: "null", response });
      }
    },
  };
}

function isTemplateInfoList(value: unknown): value is readonly TemplateInfo[] {
  return Array.isArray(value) && value.every(isTemplateInfo);
}

function isTemplateInfo(value: unknown): value is TemplateInfo {
  return (
    typeof readOwnField({ value, field: "id" }) === "string" &&
    typeof readOwnField({ value, field: "label" }) === "string"
  );
}
