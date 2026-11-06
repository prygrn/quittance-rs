import type { ReceiptScreenModel } from "./receipt-screen-model";

/** Une fiche s'enregistre dès qu'un modèle existe, hors chargement, enregistrement ou question en cours. */
export function canSaveTenantProfile(model: ReceiptScreenModel): boolean {
  const { status } = model.tenantProfileState;
  return (
    model.templates.length > 0 &&
    status !== "loading" &&
    status !== "saving" &&
    status !== "confirmingReplacement"
  );
}
