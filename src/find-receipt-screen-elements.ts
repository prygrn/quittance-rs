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
  const find = <T extends Element>(lookup: Omit<ElementLookup<T>, "root">): T =>
    requireElement({ root, ...lookup });
  const byField = <T extends Element>(lookup: {
    readonly selectorOf: (id: string) => string;
    readonly type: new () => T;
  }): Record<ReceiptFormField, T> =>
    mapFields((field) =>
      find({ selector: lookup.selectorOf(RECEIPT_FIELD_IDS[field]), type: lookup.type }),
    );

  return {
    form: find({ selector: "#receipt-form", type: HTMLFormElement }),
    formControls: find({ selector: "#form-controls", type: HTMLFieldSetElement }),
    templateSelect: find({ selector: "#template", type: HTMLSelectElement }),
    fieldInputs: mapFields((field) => requireTextField({ root, id: RECEIPT_FIELD_IDS[field] })),
    fieldErrorMessages: byField({
      selectorOf: (id) => `#${id}${FIELD_ERROR_ID_SUFFIX}`,
      type: HTMLElement,
    }),
    fieldErrorTexts: byField({
      selectorOf: (id) => `#${id}${FIELD_ERROR_ID_SUFFIX} .field-error-text`,
      type: HTMLElement,
    }),
    total: find({ selector: "#total", type: HTMLOutputElement }),
    announcement: find({ selector: "#announcement", type: HTMLElement }),
    errorSummary: find({ selector: "#error-summary", type: HTMLElement }),
    errorSummaryText: find({ selector: "#error-summary .banner-text", type: HTMLElement }),
    staleBanner: find({ selector: "#stale-banner", type: HTMLElement }),
    sendSuccess: find({ selector: "#send-success", type: HTMLElement }),
    sendSuccessRecipient: find({ selector: "#send-success .banner-recipient", type: HTMLElement }),
    sendFailure: find({ selector: "#send-failure", type: HTMLElement }),
    sendFailureTitle: find({ selector: "#send-failure .banner-title", type: HTMLElement }),
    sendFailureText: find({ selector: "#send-failure .banner-text", type: HTMLElement }),
    recipientLine: find({ selector: "#recipient-line", type: HTMLElement }),
    recipient: find({ selector: "#recipient-line .recipient", type: HTMLElement }),
    previewButton: find({ selector: "#preview-button", type: HTMLButtonElement }),
    sendButton: find({ selector: "#send-button", type: HTMLButtonElement }),
    sendButtonSpinner: find({ selector: "#send-button .spinner", type: HTMLElement }),
    sendButtonLabel: find({ selector: "#send-button .button-label", type: HTMLElement }),
    retryButton: find({ selector: "#retry-button", type: HTMLButtonElement }),
    previewStatus: find({ selector: "#preview-status", type: HTMLElement }),
    previewStatusLabel: find({
      selector: "#preview-status .preview-status-label",
      type: HTMLElement,
    }),
    previewPane: find({ selector: "#preview-pane", type: HTMLElement }),
    previewEmpty: find({ selector: "#preview-empty", type: HTMLElement }),
    previewEmptyTitle: find({ selector: "#preview-empty .pane-message-title", type: HTMLElement }),
    previewEmptyText: find({ selector: "#preview-empty .pane-message-text", type: HTMLElement }),
    previewLoading: find({ selector: "#preview-loading", type: HTMLElement }),
    previewFailure: find({ selector: "#preview-failure", type: HTMLElement }),
    previewFailureTitle: find({
      selector: "#preview-failure .pane-failure-title",
      type: HTMLElement,
    }),
    previewFailureText: find({
      selector: "#preview-failure .pane-failure-text",
      type: HTMLElement,
    }),
    previewFailureHint: find({
      selector: "#preview-failure .pane-failure-hint",
      type: HTMLElement,
    }),
    previewPage: find({ selector: "#preview-page", type: HTMLElement }),
    previewFrame: find({ selector: "#preview-frame", type: HTMLIFrameElement }),
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
