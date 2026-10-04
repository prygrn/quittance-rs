import type { ReceiptFormField } from "./receipt-form";

/** Éléments de `index.html` que l'écran lit ou met à jour. */
export interface ReceiptScreenElements {
  readonly form: HTMLFormElement;
  /** Regroupe tous les champs et le choix du modèle, désactivés d'un bloc pendant l'envoi. */
  readonly formControls: HTMLFieldSetElement;
  readonly templateSelect: HTMLSelectElement;
  readonly fieldInputs: Readonly<Record<ReceiptFormField, HTMLInputElement | HTMLTextAreaElement>>;
  readonly fieldErrorMessages: Readonly<Record<ReceiptFormField, HTMLElement>>;
  readonly fieldErrorTexts: Readonly<Record<ReceiptFormField, HTMLElement>>;
  readonly total: HTMLOutputElement;
  readonly announcement: HTMLElement;
  readonly errorSummary: HTMLElement;
  readonly errorSummaryText: HTMLElement;
  readonly staleBanner: HTMLElement;
  readonly sendSuccess: HTMLElement;
  readonly sendSuccessRecipient: HTMLElement;
  readonly sendFailure: HTMLElement;
  readonly sendFailureTitle: HTMLElement;
  readonly sendFailureText: HTMLElement;
  readonly recipientLine: HTMLElement;
  readonly recipient: HTMLElement;
  readonly previewButton: HTMLButtonElement;
  readonly sendButton: HTMLButtonElement;
  readonly sendButtonSpinner: HTMLElement;
  readonly sendButtonLabel: HTMLElement;
  readonly retryButton: HTMLButtonElement;
  readonly previewStatus: HTMLElement;
  readonly previewStatusLabel: HTMLElement;
  readonly previewPane: HTMLElement;
  readonly previewEmpty: HTMLElement;
  readonly previewEmptyTitle: HTMLElement;
  readonly previewEmptyText: HTMLElement;
  readonly previewLoading: HTMLElement;
  readonly previewFailure: HTMLElement;
  readonly previewFailureTitle: HTMLElement;
  readonly previewFailureText: HTMLElement;
  readonly previewFailureHint: HTMLElement;
  readonly previewPage: HTMLElement;
  readonly previewFrame: HTMLIFrameElement;
}
