import type { CommandErrorCode, ReceiptInput } from "./api";

// `previewing` couvre l'aperçu demandé (`previewHtml: null`) puis reçu ; le HTML est
// affiché par l'UI dans une `iframe srcdoc`. `error.sendRetry` garde la saisie et son
// aperçu quand l'envoi a échoué, pour relancer l'envoi sans nouvel aperçu.
export type ScreenState =
  | { status: "idle" }
  | { status: "previewing"; input: ReceiptInput; templateId: string; previewHtml: string | null }
  | ({ status: "sending" } & PreviewedReceipt)
  | ({ status: "sent" } & PreviewedReceipt)
  | { status: "error"; code: CommandErrorCode; sendRetry: PreviewedReceipt | null };

/** Un aperçu dépend de la saisie et du modèle choisi. */
export interface PreviewRequest {
  input: ReceiptInput;
  templateId: string;
}

/** Saisie prévisualisée avec son aperçu, seule forme qu'un envoi accepte. */
export interface PreviewedReceipt {
  input: ReceiptInput;
  templateId: string;
  previewHtml: string;
}

// `previewRequested` exige un `ReceiptInput`, que seule une saisie validée produit.
// Les réponses d'aperçu rappellent la saisie et le modèle demandés, pour écarter les réponses tardives.
export type ScreenEvent =
  | ({ type: "previewRequested" } & PreviewRequest)
  | ({ type: "previewReceived"; html: string } & PreviewRequest)
  | ({ type: "previewFailed"; code: CommandErrorCode } & PreviewRequest)
  | { type: "sendRequested" }
  | { type: "sendSucceeded" }
  | { type: "sendFailed"; code: CommandErrorCode }
  | { type: "formEdited" };

export const INITIAL_SCREEN_STATE: ScreenState = { status: "idle" };

/** Transition pure de l'écran ; une transition invalide renvoie l'état inchangé. */
export function nextScreenState(state: ScreenState, event: ScreenEvent): ScreenState {
  throw new Error("not implemented", { cause: [state, event] });
}
