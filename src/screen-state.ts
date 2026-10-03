import type { CommandErrorCode, ReceiptInput } from "./api";

export type ScreenState =
  | { status: "idle" }
  | { status: "previewing"; input: ReceiptInput; previewHtml: string | null }
  | { status: "sending"; input: ReceiptInput; previewHtml: string }
  | { status: "sent"; input: ReceiptInput; previewHtml: string }
  | { status: "error"; code: CommandErrorCode; sendRetry: PreviewedReceipt | null };

/** Saisie prévisualisée avec son aperçu, seule forme qu'un envoi accepte. */
export interface PreviewedReceipt {
  input: ReceiptInput;
  previewHtml: string;
}

export type ScreenEvent =
  | { type: "previewRequested"; input: ReceiptInput }
  | { type: "previewReceived"; input: ReceiptInput; html: string }
  | { type: "previewFailed"; input: ReceiptInput; code: CommandErrorCode }
  | { type: "sendRequested" }
  | { type: "sendSucceeded" }
  | { type: "sendFailed"; code: CommandErrorCode }
  | { type: "formEdited" };

export const INITIAL_SCREEN_STATE: ScreenState = { status: "idle" };

/** Transition pure de l'écran ; une transition invalide renvoie l'état inchangé. */
export function nextScreenState(state: ScreenState, event: ScreenEvent): ScreenState {
  throw new Error("not implemented", { cause: [state, event] });
}
