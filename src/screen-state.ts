import type { ReceiptInput } from "./api";

export type ScreenState =
  | { status: "idle" }
  | { status: "previewing"; input: ReceiptInput; isPreviewReady: boolean }
  | { status: "sending"; input: ReceiptInput }
  | { status: "sent"; input: ReceiptInput }
  | { status: "error"; message: string; previewedInput: ReceiptInput | null };

export type ScreenEvent =
  | { type: "previewRequested"; input: ReceiptInput }
  | { type: "previewReceived" }
  | { type: "sendRequested" }
  | { type: "sendSucceeded" }
  | { type: "operationFailed"; message: string }
  | { type: "formEdited" };

export const INITIAL_SCREEN_STATE: ScreenState = { status: "idle" };

/** Transition pure de l'écran ; une transition invalide renvoie l'état inchangé. */
export function nextScreenState(state: ScreenState, event: ScreenEvent): ScreenState {
  throw new Error("not implemented", { cause: [state, event] });
}
