import type { ScreenState } from "./screen-state";

/** Vrai entre la demande d'un aperçu et sa réponse. */
export function isPreviewPending(state: ScreenState): boolean {
  return state.status === "previewing" && state.previewHtml === null;
}
