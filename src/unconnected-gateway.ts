import type { ReceiptGateway } from "./receipt-gateway";

function notConnected(): Promise<never> {
  return Promise.reject(
    new Error("[UI_BACKEND_NOT_CONNECTED] the Tauri commands are not wired to the screen yet"),
  );
}

/** Passerelle d'attente tant que les commandes Tauri ne sont pas branchées : tout appel échoue. */
export const UNCONNECTED_GATEWAY: ReceiptGateway = {
  listTemplates: notConnected,
  renderPreview: notConnected,
  sendReceipt: notConnected,
};
