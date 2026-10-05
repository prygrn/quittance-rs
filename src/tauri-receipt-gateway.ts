import type { InvokeCommand } from "./invoke-command";
import type { ReceiptGateway } from "./receipt-gateway";

/**
 * Passerelle vers les commandes Tauri `list_templates`, `preview_receipt` et
 * `send_receipt`. `now` fournit l'instant d'appel, dont la date locale devient l'`issueDate`.
 */
export function createTauriReceiptGateway(dependencies: {
  readonly invokeCommand: InvokeCommand;
  readonly now: () => Date;
}): ReceiptGateway {
  throw new Error(`[UI_NOT_IMPLEMENTED] createTauriReceiptGateway(${typeof dependencies})`);
}
