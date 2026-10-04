import { parseAmountCents } from "./amount";
import type { CommandErrorCode } from "./api";
import { canRequestPreview } from "./can-request-preview";
import { COMMAND_ERROR_MESSAGES } from "./command-error-messages";
import { isWellFormedEmail } from "./email";
import { isPreviewPending } from "./is-preview-pending";
import { FIELD_ERROR_MESSAGES } from "./field-error-messages";
import { formatEuroCents } from "./format-euro-cents";
import type { ReceiptFormField, ReceiptFormValues } from "./receipt-form";
import type { ReceiptScreenElements } from "./receipt-screen-elements";
import type { ReceiptScreenModel } from "./receipt-screen-model";
import type { ScreenState } from "./screen-state";

type StatusTone = "neutral" | "busy" | "success" | "error";

interface PreviewStatus {
  readonly label: string;
  readonly tone: StatusTone;
}

// Textes de l'écran repris de la maquette ; les messages d'erreur ont leurs propres modules.
const SCREEN_TEXTS = {
  MISSING_TOTAL: "—",
  SEND: "Envoyer",
  SENDING: "Envoi en cours…",
  EMPTY_TITLE: "Aucun aperçu pour le moment",
  EMPTY_TEXT:
    "Remplissez le formulaire puis cliquez sur « Aperçu » pour vérifier la quittance avant de l'envoyer.",
  STALE_TITLE: "L'aperçu n'est plus à jour",
  STALE_TEXT:
    "Le formulaire ou le modèle a changé. Cliquez sur « Aperçu » pour régénérer la quittance avant de l'envoyer.",
  ANNOUNCE_PREVIEW_PENDING: "Génération de l'aperçu en cours.",
  ANNOUNCE_PREVIEW_READY: "Aperçu prêt. Vous pouvez envoyer la quittance.",
  ANNOUNCE_SENDING: "Envoi en cours.",
  ANNOUNCE_STALE: "Aperçu invalidé : relancez l'aperçu.",
  announceSent: (recipient: string): string => `Quittance envoyée à ${recipient}.`,
  errorSummary: (count: number): string =>
    count === 1
      ? "1 champ est à corriger avant l'aperçu."
      : `${count} champs sont à corriger avant l'aperçu.`,
} as const;

const PREVIEW_STATUSES = {
  NOT_GENERATED: { label: "Non généré", tone: "neutral" },
  STALE: { label: "À régénérer", tone: "neutral" },
  GENERATING: { label: "Génération…", tone: "busy" },
  UP_TO_DATE: { label: "À jour", tone: "success" },
  SENDING: { label: "Envoi…", tone: "busy" },
  SENT: { label: "Envoyée", tone: "success" },
  PREVIEW_FAILED: { label: "Échec de l’aperçu", tone: "error" },
  SEND_FAILED: { label: "À jour · envoi échoué", tone: "error" },
} as const satisfies Record<string, PreviewStatus>;

/** Reporte le modèle de l'écran et la saisie courante sur le DOM. */
export function renderReceiptScreen(options: {
  readonly elements: ReceiptScreenElements;
  readonly model: ReceiptScreenModel;
  readonly formValues: ReceiptFormValues;
}): void {
  const { elements, model, formValues } = options;
  const { screenState } = model;
  const isSending = screenState.status === "sending";
  const isWaitingForPreview = isPreviewPending(screenState);
  const isIdleWithoutErrors = screenState.status === "idle" && model.fieldErrors === null;
  const previewHtml = displayedPreviewHtml(screenState);
  const sendFailureCode =
    screenState.status === "error" && screenState.sendRetry !== null ? screenState.code : null;
  const paneFailureCode = paneFailureCodeOf(model);
  const recipient = formValues.tenantEmail.trim();

  elements.formControls.disabled = isSending;
  elements.form.setAttribute("aria-busy", String(isSending));
  renderFieldErrors({ elements, model });
  setText({ element: elements.total, text: totalOf(formValues) });
  setText({ element: elements.announcement, text: announcementOf(model) });

  const errorCount = Object.keys(model.fieldErrors ?? {}).length;
  elements.errorSummary.hidden = errorCount === 0;
  setText({
    element: elements.errorSummaryText,
    text: errorCount === 0 ? "" : SCREEN_TEXTS.errorSummary(errorCount),
  });
  elements.staleBanner.hidden = !(isIdleWithoutErrors && model.isPreviewStale);
  elements.sendSuccess.hidden = screenState.status !== "sent";
  setText({
    element: elements.sendSuccessRecipient,
    text: screenState.status === "sent" ? screenState.input.tenantEmail : "",
  });
  elements.sendFailure.hidden = sendFailureCode === null;
  setText({ element: elements.sendFailureTitle, text: failureTitle(sendFailureCode) });
  setText({ element: elements.sendFailureText, text: failureText(sendFailureCode) });

  const hasRecipient =
    isWellFormedEmail(recipient) &&
    (previewHtml !== null || (isIdleWithoutErrors && model.isPreviewStale)) &&
    screenState.status !== "sent";
  elements.recipientLine.hidden = !hasRecipient;
  setText({ element: elements.recipient, text: hasRecipient ? recipient : "" });

  elements.previewButton.disabled = !canRequestPreview(model);
  elements.previewButton.setAttribute("aria-busy", String(isWaitingForPreview));
  elements.sendButton.hidden = sendFailureCode !== null;
  elements.sendButton.disabled = !(screenState.status === "previewing" && previewHtml !== null);
  elements.sendButton.setAttribute("aria-busy", String(isSending));
  elements.sendButtonSpinner.hidden = !isSending;
  setText({
    element: elements.sendButtonLabel,
    text: isSending ? SCREEN_TEXTS.SENDING : SCREEN_TEXTS.SEND,
  });
  elements.retryButton.hidden = sendFailureCode === null;

  const status = previewStatusOf(model);
  elements.previewStatus.dataset["tone"] = status.tone;
  setText({ element: elements.previewStatusLabel, text: status.label });

  elements.previewPane.setAttribute("aria-busy", String(isWaitingForPreview));
  elements.previewEmpty.hidden = !(screenState.status === "idle" && paneFailureCode === null);
  setText({
    element: elements.previewEmptyTitle,
    text: model.isPreviewStale ? SCREEN_TEXTS.STALE_TITLE : SCREEN_TEXTS.EMPTY_TITLE,
  });
  setText({
    element: elements.previewEmptyText,
    text: model.isPreviewStale ? SCREEN_TEXTS.STALE_TEXT : SCREEN_TEXTS.EMPTY_TEXT,
  });
  elements.previewLoading.hidden = !isWaitingForPreview;
  elements.previewFailure.hidden = paneFailureCode === null;
  setText({ element: elements.previewFailureTitle, text: failureTitle(paneFailureCode) });
  setText({ element: elements.previewFailureText, text: failureText(paneFailureCode) });
  // Sans modèle chargé, relancer l'aperçu ne sert à rien : le conseil est masqué.
  elements.previewFailureHint.hidden = model.templateLoadError !== null;
  elements.previewPage.hidden = previewHtml === null;
  elements.previewPage.dataset["dimmed"] = String(isSending);
  // Réécrire `srcdoc` recharge l'iframe : seul un nouvel aperçu le remplace.
  if (previewHtml !== null && elements.previewFrame.getAttribute("srcdoc") !== previewHtml) {
    elements.previewFrame.setAttribute("srcdoc", previewHtml);
  }
}

function renderFieldErrors(options: {
  readonly elements: ReceiptScreenElements;
  readonly model: ReceiptScreenModel;
}): void {
  const { elements, model } = options;
  (Object.keys(elements.fieldInputs) as ReceiptFormField[]).forEach((field) => {
    const error = model.fieldErrors?.[field];
    const input = elements.fieldInputs[field];
    const message = elements.fieldErrorMessages[field];
    input.setAttribute("aria-invalid", String(error !== undefined));
    if (error === undefined) {
      input.removeAttribute("aria-describedby");
    } else {
      input.setAttribute("aria-describedby", message.id);
    }
    message.hidden = error === undefined;
    setText({
      element: elements.fieldErrorTexts[field],
      text: error === undefined ? "" : FIELD_ERROR_MESSAGES[error],
    });
  });
}

function displayedPreviewHtml(state: ScreenState): string | null {
  switch (state.status) {
    case "previewing":
      return state.previewHtml;
    case "sending":
    case "sent":
      return state.previewHtml;
    case "error":
      return state.sendRetry?.previewHtml ?? null;
    case "idle":
      return null;
  }
}

function paneFailureCodeOf(model: ReceiptScreenModel): CommandErrorCode | null {
  const { screenState } = model;
  if (screenState.status === "error") {
    return screenState.sendRetry === null ? screenState.code : null;
  }
  return screenState.status === "idle" ? model.templateLoadError : null;
}

function totalOf(values: ReceiptFormValues): string {
  const rent = parseAmountCents(values.rentAmount);
  const charges = parseAmountCents(values.chargesAmount);
  return rent.isValid && charges.isValid
    ? formatEuroCents(rent.cents + charges.cents)
    : SCREEN_TEXTS.MISSING_TOTAL;
}

function announcementOf(model: ReceiptScreenModel): string {
  const { screenState } = model;
  switch (screenState.status) {
    case "previewing":
      return screenState.previewHtml === null
        ? SCREEN_TEXTS.ANNOUNCE_PREVIEW_PENDING
        : SCREEN_TEXTS.ANNOUNCE_PREVIEW_READY;
    case "sending":
      return SCREEN_TEXTS.ANNOUNCE_SENDING;
    case "sent":
      return SCREEN_TEXTS.announceSent(screenState.input.tenantEmail);
    case "error":
      return failureAnnouncement(screenState.code);
    case "idle":
      if (model.templateLoadError !== null) {
        return failureAnnouncement(model.templateLoadError);
      }
      return model.isPreviewStale && model.fieldErrors === null ? SCREEN_TEXTS.ANNOUNCE_STALE : "";
  }
}

function previewStatusOf(model: ReceiptScreenModel): PreviewStatus {
  const { screenState } = model;
  switch (screenState.status) {
    case "idle":
      return model.isPreviewStale ? PREVIEW_STATUSES.STALE : PREVIEW_STATUSES.NOT_GENERATED;
    case "previewing":
      return screenState.previewHtml === null
        ? PREVIEW_STATUSES.GENERATING
        : PREVIEW_STATUSES.UP_TO_DATE;
    case "sending":
      return PREVIEW_STATUSES.SENDING;
    case "sent":
      return PREVIEW_STATUSES.SENT;
    case "error":
      return screenState.sendRetry === null
        ? PREVIEW_STATUSES.PREVIEW_FAILED
        : PREVIEW_STATUSES.SEND_FAILED;
  }
}

function failureTitle(code: CommandErrorCode | null): string {
  return code === null ? "" : COMMAND_ERROR_MESSAGES[code].title;
}

function failureText(code: CommandErrorCode | null): string {
  return code === null ? "" : COMMAND_ERROR_MESSAGES[code].text;
}

function failureAnnouncement(code: CommandErrorCode): string {
  return `${failureTitle(code)}. ${failureText(code)}`;
}

// Réécrire un texte identique ferait répéter les régions `aria-live` par certains lecteurs d'écran.
function setText(options: { readonly element: HTMLElement; readonly text: string }): void {
  if (options.element.textContent !== options.text) {
    options.element.textContent = options.text;
  }
}
