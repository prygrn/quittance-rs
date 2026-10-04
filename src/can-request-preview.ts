import { isPreviewPending } from "./is-preview-pending";
import type { ReceiptScreenModel } from "./receipt-screen-model";

/** Un aperçu se demande dès qu'un modèle existe, hors envoi et hors aperçu déjà en cours. */
export function canRequestPreview(model: ReceiptScreenModel): boolean {
  const { screenState } = model;
  return (
    model.templates.length > 0 && screenState.status !== "sending" && !isPreviewPending(screenState)
  );
}
