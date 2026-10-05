import { TIMEOUTS_MS } from "./timeouts-ms";

/**
 * Attend que l'écran soit utilisable : la liste des modèles, chargée depuis le backend
 * au démarrage, propose au moins un modèle.
 */
export async function openReceiptScreen(): Promise<void> {
  await $("#template option").waitForExist({
    timeout: TIMEOUTS_MS.templates,
    timeoutMsg: "[E2E_SCREEN_NOT_READY] the template list was never filled by the backend",
  });
}
