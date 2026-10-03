import type { ReceiptInput } from "./api";

// `previewing` couvre l'aperçu demandé puis reçu (`isPreviewReady`) ; le contenu de
// l'aperçu reste hors de l'état, son type sera figé par la couche Tauri (F6).
// `error.previewedInput` garde la saisie déjà prévisualisée quand l'envoi a échoué,
// pour relancer l'envoi sans nouvel aperçu.
export type ScreenState =
  | { status: "idle" }
  | { status: "previewing"; input: ReceiptInput; isPreviewReady: boolean }
  | { status: "sending"; input: ReceiptInput }
  | { status: "sent"; input: ReceiptInput }
  | { status: "error"; message: string; previewedInput: ReceiptInput | null };

// `previewRequested` exige un `ReceiptInput`, que seule une saisie validée produit.
export type ScreenEvent =
  | { type: "previewRequested"; input: ReceiptInput }
  | { type: "previewReceived" }
  | { type: "sendRequested" }
  | { type: "sendSucceeded" }
  | { type: "operationFailed"; message: string }
  | { type: "formEdited" };

export const INITIAL_SCREEN_STATE: ScreenState = { status: "idle" };

/**
 * Transition pure de l'écran ; une transition invalide renvoie l'état inchangé
 * (même référence). Ignorer plutôt que lever neutralise les réponses asynchrones
 * tardives, comme un aperçu reçu après une modification du formulaire.
 */
export function nextScreenState(state: ScreenState, event: ScreenEvent): ScreenState {
  switch (state.status) {
    case "idle":
      return nextIdleState(state, event);
    case "previewing":
      return nextPreviewingState(state, event);
    case "sending":
      return nextSendingState(state, event);
    case "sent":
      return nextSentState(state, event);
    case "error":
      return nextErrorState(state, event);
  }
}

type StateOf<Status extends ScreenState["status"]> = Extract<ScreenState, { status: Status }>;

function pendingPreviewState(input: ReceiptInput): ScreenState {
  return { status: "previewing", input, isPreviewReady: false };
}

function nextIdleState(state: StateOf<"idle">, event: ScreenEvent): ScreenState {
  return event.type === "previewRequested" ? pendingPreviewState(event.input) : state;
}

function nextPreviewingState(state: StateOf<"previewing">, event: ScreenEvent): ScreenState {
  switch (event.type) {
    case "previewRequested":
      return pendingPreviewState(event.input);
    case "formEdited":
      return INITIAL_SCREEN_STATE;
    case "previewReceived":
      return state.isPreviewReady ? state : { ...state, isPreviewReady: true };
    case "operationFailed":
      return state.isPreviewReady
        ? state
        : { status: "error", message: event.message, previewedInput: null };
    case "sendRequested":
      return state.isPreviewReady ? { status: "sending", input: state.input } : state;
    case "sendSucceeded":
      return state;
  }
}

// Le formulaire est verrouillé pendant l'envoi : `formEdited` y est ignoré pour ne
// pas perdre de vue un envoi qui aboutirait ensuite.
function nextSendingState(state: StateOf<"sending">, event: ScreenEvent): ScreenState {
  switch (event.type) {
    case "sendSucceeded":
      return { status: "sent", input: state.input };
    case "operationFailed":
      return { status: "error", message: event.message, previewedInput: state.input };
    default:
      return state;
  }
}

function nextSentState(state: StateOf<"sent">, event: ScreenEvent): ScreenState {
  switch (event.type) {
    case "previewRequested":
      return pendingPreviewState(event.input);
    case "formEdited":
      return INITIAL_SCREEN_STATE;
    default:
      return state;
  }
}

function nextErrorState(state: StateOf<"error">, event: ScreenEvent): ScreenState {
  switch (event.type) {
    case "previewRequested":
      return pendingPreviewState(event.input);
    case "formEdited":
      return INITIAL_SCREEN_STATE;
    case "sendRequested":
      return state.previewedInput === null
        ? state
        : { status: "sending", input: state.previewedInput };
    default:
      return state;
  }
}
