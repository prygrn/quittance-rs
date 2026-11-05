import { canSaveTenantProfile } from "./can-save-tenant-profile";
import { COMMAND_ERROR_MESSAGES } from "./command-error-messages";
import type { ReceiptScreenElements } from "./receipt-screen-elements";
import type { ReceiptScreenModel } from "./receipt-screen-model";
import { setText } from "./set-text";

/** Reporte l'état des fiches locataire sur leur liste, leur bouton et leurs bandeaux. */
export function renderTenantProfileControls(options: {
  readonly elements: ReceiptScreenElements;
  readonly model: ReceiptScreenModel;
}): void {
  const { elements, model } = options;
  const state = model.tenantProfileState;
  const isSaving = state.status === "saving";

  elements.tenantProfileSelect.disabled = model.templates.length === 0 || isSaving;
  elements.saveTenantProfileButton.disabled = !canSaveTenantProfile(model);
  elements.saveTenantProfileButton.setAttribute("aria-busy", String(isSaving));

  elements.tenantProfileReplacement.hidden = state.status !== "confirmingReplacement";
  setText({
    element: elements.tenantProfileReplacementName,
    text: state.status === "confirmingReplacement" ? state.profile.tenantName : "",
  });
  elements.tenantProfileSaved.hidden = state.status !== "saved";
  setText({
    element: elements.tenantProfileSavedLabel,
    text: state.status === "saved" ? state.entry.label : "",
  });
  const failure = state.status === "failed" ? COMMAND_ERROR_MESSAGES[state.code] : null;
  elements.tenantProfileFailure.hidden = failure === null;
  setText({ element: elements.tenantProfileFailureTitle, text: failure?.title ?? "" });
  setText({ element: elements.tenantProfileFailureText, text: failure?.text ?? "" });
}
