import type { ReceiptFormField } from "./receipt-form";
import type { ReceiptScreenElements } from "./receipt-screen-elements";

// Identifiants stables du balisage, repris par les tests et les futurs tests e2e.
const RECEIPT_FIELD_IDS: Readonly<Record<ReceiptFormField, string>> = {
  tenantName: "tenant-name",
  tenantAddress: "tenant-address",
  tenantEmail: "tenant-email",
  propertyAddress: "property-address",
  periodStart: "period-start",
  periodEnd: "period-end",
  rentAmount: "rent-amount",
  chargesAmount: "charges-amount",
  paymentDate: "payment-date",
};

const FIELD_ERROR_ID_SUFFIX = "-error";

interface ElementLookup<T extends Element> {
  readonly root: ParentNode;
  readonly selector: string;
  readonly type: new () => T;
}

/**
 * Retrouve les éléments de l'écran dans `root` ; lève une erreur explicite si le balisage
 * de `index.html` ne correspond plus à ce que l'écran attend.
 */
export function findReceiptScreenElements(root: ParentNode): ReceiptScreenElements {
  const find = <T extends Element>(selector: string, type: new () => T): T =>
    requireElement({ root, selector, type });
  const byField = <T extends Element>(
    selectorOf: (id: string) => string,
    type: new () => T,
  ): Record<ReceiptFormField, T> =>
    mapFields((field) => find(selectorOf(RECEIPT_FIELD_IDS[field]), type));

  return {
    form: find("#receipt-form", HTMLFormElement),
    formControls: find("#form-controls", HTMLFieldSetElement),
    templateSelect: find("#template", HTMLSelectElement),
    fieldInputs: mapFields((field) => requireTextField({ root, id: RECEIPT_FIELD_IDS[field] })),
    fieldErrorMessages: byField((id) => `#${id}${FIELD_ERROR_ID_SUFFIX}`, HTMLElement),
    fieldErrorTexts: byField(
      (id) => `#${id}${FIELD_ERROR_ID_SUFFIX} .field-error-text`,
      HTMLElement,
    ),
    total: find("#total", HTMLOutputElement),
    announcement: find("#announcement", HTMLElement),
    errorSummary: find("#error-summary", HTMLElement),
    errorSummaryText: find("#error-summary .banner-text", HTMLElement),
    staleBanner: find("#stale-banner", HTMLElement),
    sendSuccess: find("#send-success", HTMLElement),
    sendSuccessRecipient: find("#send-success .banner-recipient", HTMLElement),
    sendFailure: find("#send-failure", HTMLElement),
    sendFailureTitle: find("#send-failure .banner-title", HTMLElement),
    sendFailureText: find("#send-failure .banner-text", HTMLElement),
    recipientLine: find("#recipient-line", HTMLElement),
    recipient: find("#recipient-line .recipient", HTMLElement),
    previewButton: find("#preview-button", HTMLButtonElement),
    sendButton: find("#send-button", HTMLButtonElement),
    sendButtonSpinner: find("#send-button .spinner", HTMLElement),
    sendButtonLabel: find("#send-button .button-label", HTMLElement),
    retryButton: find("#retry-button", HTMLButtonElement),
    previewStatus: find("#preview-status", HTMLElement),
    previewStatusLabel: find("#preview-status .preview-status-label", HTMLElement),
    previewPane: find("#preview-pane", HTMLElement),
    previewEmpty: find("#preview-empty", HTMLElement),
    previewEmptyTitle: find("#preview-empty .pane-message-title", HTMLElement),
    previewEmptyText: find("#preview-empty .pane-message-text", HTMLElement),
    previewLoading: find("#preview-loading", HTMLElement),
    previewFailure: find("#preview-failure", HTMLElement),
    previewFailureTitle: find("#preview-failure .pane-failure-title", HTMLElement),
    previewFailureText: find("#preview-failure .pane-failure-text", HTMLElement),
    previewFailureHint: find("#preview-failure .pane-failure-hint", HTMLElement),
    previewPage: find("#preview-page", HTMLElement),
    previewFrame: find("#preview-frame", HTMLIFrameElement),
  };
}

function mapFields<T>(valueOf: (field: ReceiptFormField) => T): Record<ReceiptFormField, T> {
  const fields = Object.keys(RECEIPT_FIELD_IDS) as ReceiptFormField[];
  return Object.fromEntries(fields.map((field) => [field, valueOf(field)])) as Record<
    ReceiptFormField,
    T
  >;
}

function requireElement<T extends Element>(lookup: ElementLookup<T>): T {
  const element = lookup.root.querySelector(lookup.selector);
  if (!(element instanceof lookup.type)) {
    throw new Error(
      `[UI_MARKUP_MISMATCH] receipt screen expects ${lookup.selector} to be a ${lookup.type.name}, found ${element?.tagName ?? "nothing"}`,
    );
  }
  return element;
}

function requireTextField(options: {
  readonly root: ParentNode;
  readonly id: string;
}): HTMLInputElement | HTMLTextAreaElement {
  const element = requireElement({
    root: options.root,
    selector: `#${options.id}`,
    type: HTMLElement,
  });
  if (!(element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement)) {
    throw new Error(
      `[UI_MARKUP_MISMATCH] receipt screen expects #${options.id} to be an input or a textarea, found ${element.tagName}`,
    );
  }
  return element;
}
