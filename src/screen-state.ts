import type { CommandErrorCode, ReceiptInput } from "./api";

// `previewing` couvre l'aperçu demandé (`previewHtml: null`) puis reçu ; le HTML est
// affiché par l'UI dans une `iframe srcdoc`. `error.sendRetry` garde la saisie et son
// aperçu quand l'envoi a échoué, pour relancer l'envoi sans nouvel aperçu.
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

// `previewRequested` exige un `ReceiptInput`, que seule une saisie validée produit.
// Les réponses d'aperçu rappellent la saisie demandée, pour écarter les réponses tardives.
export type ScreenEvent =
  | { type: "previewRequested"; input: ReceiptInput }
  | { type: "previewReceived"; input: ReceiptInput; html: string }
  | { type: "previewFailed"; input: ReceiptInput; code: CommandErrorCode }
  | { type: "sendRequested" }
  | { type: "sendSucceeded" }
  | { type: "sendFailed"; code: CommandErrorCode }
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
      return event.type === "previewRequested" ? pendingPreviewState(event.input) : state;
    case "previewing":
      return nextPreviewingState(state, event);
    case "sending":
      return nextSendingState(state, event);
    case "sent":
      return nextSettledState(state, event);
    case "error":
      return event.type === "sendRequested" && state.sendRetry !== null
        ? { status: "sending", ...state.sendRetry }
        : nextSettledState(state, event);
  }
}

type StateOf<Status extends ScreenState["status"]> = Extract<ScreenState, { status: Status }>;

function pendingPreviewState(input: ReceiptInput): ScreenState {
  return { status: "previewing", input, previewHtml: null };
}

function nextPreviewingState(state: StateOf<"previewing">, event: ScreenEvent): ScreenState {
  const isPreviewPending = state.previewHtml === null;
  switch (event.type) {
    case "previewRequested":
      return pendingPreviewState(event.input);
    case "formEdited":
      return INITIAL_SCREEN_STATE;
    case "previewReceived":
      return isPreviewPending && isSameReceiptInput(event.input, state.input)
        ? { ...state, previewHtml: event.html }
        : state;
    case "previewFailed":
      return isPreviewPending && isSameReceiptInput(event.input, state.input)
        ? { status: "error", code: event.code, sendRetry: null }
        : state;
    case "sendRequested":
      return state.previewHtml === null
        ? state
        : { status: "sending", input: state.input, previewHtml: state.previewHtml };
    default:
      return state;
  }
}

// F7c DOIT désactiver le formulaire pendant `sending` : la machine ignore donc
// `formEdited` dans cet état, sans quoi l'issue d'un envoi en cours serait perdue.
function nextSendingState(state: StateOf<"sending">, event: ScreenEvent): ScreenState {
  switch (event.type) {
    case "sendSucceeded":
      return { ...state, status: "sent" };
    case "sendFailed":
      return {
        status: "error",
        code: event.code,
        sendRetry: { input: state.input, previewHtml: state.previewHtml },
      };
    default:
      return state;
  }
}

/** États sans opération en cours : seule une nouvelle saisie ou un nouvel aperçu les quitte. */
function nextSettledState(state: StateOf<"sent" | "error">, event: ScreenEvent): ScreenState {
  switch (event.type) {
    case "previewRequested":
      return pendingPreviewState(event.input);
    case "formEdited":
      return INITIAL_SCREEN_STATE;
    default:
      return state;
  }
}

/**
 * Égalité structurelle plutôt que par référence : l'invariant protégé porte sur le
 * contenu prévisualisé, et tous les champs de `ReceiptInput` sont des primitives.
 * L'UI n'a ainsi pas à conserver l'objet exact passé à la requête.
 */
function isSameReceiptInput(left: ReceiptInput, right: ReceiptInput): boolean {
  return (Object.keys(right) as (keyof ReceiptInput)[]).every(
    (field) => left[field] === right[field],
  );
}
