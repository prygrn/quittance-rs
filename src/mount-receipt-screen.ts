import { canRequestPreview } from "./can-request-preview";
import { findReceiptScreenElements } from "./find-receipt-screen-elements";
import { frenchDateToIsoDate } from "./french-date-to-iso-date";
import {
  type ReceiptFormResult,
  type ReceiptFormValues,
  validateReceiptForm,
} from "./receipt-form";
import type { ReceiptGateway } from "./receipt-gateway";
import type { ReceiptScreenModel } from "./receipt-screen-model";
import { readReceiptFormValues } from "./read-receipt-form-values";
import { renderReceiptScreen } from "./render-receipt-screen";
import {
  INITIAL_SCREEN_STATE,
  nextScreenState,
  type PreviewRequest,
  type ScreenEvent,
} from "./screen-state";
import { toCommandErrorCode } from "./to-command-error-code";

const LOG_CODES = {
  TEMPLATE_LISTING_FAILED: "UI_TEMPLATE_LISTING_FAILED",
  NO_TEMPLATE: "UI_NO_TEMPLATE",
  PREVIEW_FAILED: "UI_PREVIEW_FAILED",
  SEND_FAILED: "UI_SEND_FAILED",
} as const;

/**
 * Branche l'écran de quittance sur le balisage de `index.html` présent dans `root`,
 * puis charge la liste des modèles. La promesse se résout une fois cette liste affichée.
 */
export async function mountReceiptScreen(options: {
  readonly root: ParentNode;
  readonly gateway: ReceiptGateway;
}): Promise<void> {
  const { gateway } = options;
  const elements = findReceiptScreenElements(options.root);
  let model: ReceiptScreenModel = {
    screenState: INITIAL_SCREEN_STATE,
    fieldErrors: null,
    isPreviewStale: false,
    templates: [],
    templateLoadError: null,
  };

  const readValues = (): ReceiptFormValues => readReceiptFormValues(elements.fieldInputs);
  const update = (changes: Partial<ReceiptScreenModel>): void => {
    model = { ...model, ...changes };
    renderReceiptScreen({ elements, model, formValues: readValues() });
  };
  const dispatch = (event: ScreenEvent): void => {
    update({ screenState: nextScreenState(model.screenState, event) });
  };

  const handleEdit = (): void => {
    // Défense : le formulaire est désactivé pendant l'envoi, dont l'issue ne doit pas se perdre.
    if (model.screenState.status === "sending") {
      return;
    }
    const validation = model.fieldErrors === null ? null : validateScreenForm(readValues());
    update({
      screenState: nextScreenState(model.screenState, { type: "formEdited" }),
      isPreviewStale: model.screenState.status !== "idle" || model.isPreviewStale,
      fieldErrors: validation === null ? null : validation.isValid ? {} : validation.errors,
    });
  };

  const requestPreview = (): void => {
    if (!canRequestPreview(model)) {
      return;
    }
    const validation = validateScreenForm(readValues());
    if (!validation.isValid) {
      update({ fieldErrors: validation.errors, isPreviewStale: false });
      elements.form.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
      return;
    }
    const request: PreviewRequest = {
      input: validation.input,
      templateId: elements.templateSelect.value,
    };
    model = { ...model, fieldErrors: null, isPreviewStale: false };
    dispatch({ type: "previewRequested", ...request });
    gateway.renderPreview(request).then(
      (html: string) => dispatch({ type: "previewReceived", ...request, html }),
      (reason: unknown) => {
        console.error(`${LOG_CODES.PREVIEW_FAILED}: preview command failed`, reason);
        dispatch({ type: "previewFailed", ...request, code: toCommandErrorCode(reason) });
      },
    );
  };

  const requestSend = (): void => {
    const previousState = model.screenState;
    dispatch({ type: "sendRequested" });
    const state = model.screenState;
    if (state === previousState || state.status !== "sending") {
      return;
    }
    const request: PreviewRequest = { input: state.input, templateId: state.templateId };
    gateway.sendReceipt(request).then(
      () => dispatch({ type: "sendSucceeded" }),
      (reason: unknown) => {
        console.error(`${LOG_CODES.SEND_FAILED}: send command failed`, reason);
        dispatch({ type: "sendFailed", code: toCommandErrorCode(reason) });
      },
    );
  };

  // `input` couvre aussi la liste des modèles : un changement de modèle invalide l'aperçu.
  elements.form.addEventListener("input", handleEdit);
  elements.form.addEventListener("submit", (event: SubmitEvent) => {
    event.preventDefault();
    requestPreview();
  });
  elements.sendButton.addEventListener("click", requestSend);
  elements.retryButton.addEventListener("click", requestSend);
  update({});

  try {
    const templates = await gateway.listTemplates();
    if (templates.length === 0) {
      console.error(`${LOG_CODES.NO_TEMPLATE}: the backend lists no receipt template`);
      update({ templateLoadError: "template" });
      return;
    }
    const document = elements.templateSelect.ownerDocument;
    elements.templateSelect.replaceChildren(
      ...templates.map((template) => {
        const option = document.createElement("option");
        option.value = template.id;
        option.textContent = template.label;
        return option;
      }),
    );
    update({ templates });
  } catch (reason: unknown) {
    console.error(`${LOG_CODES.TEMPLATE_LISTING_FAILED}: template listing command failed`, reason);
    update({ templateLoadError: toCommandErrorCode(reason) });
  }
}

// Les dates se saisissent en jj/mm/aaaa (maquette) ; `validateReceiptForm` attend de l'ISO.
function validateScreenForm(values: ReceiptFormValues): ReceiptFormResult {
  return validateReceiptForm({
    ...values,
    periodStart: frenchDateToIsoDate(values.periodStart),
    periodEnd: frenchDateToIsoDate(values.periodEnd),
    paymentDate: frenchDateToIsoDate(values.paymentDate),
  });
}
