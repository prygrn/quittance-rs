import type { CommandErrorCode, TenantProfileEntry, TenantProfilePayload } from "./api";
import { canRequestPreview } from "./can-request-preview";
import { canSaveTenantProfile } from "./can-save-tenant-profile";
import { createOption } from "./create-option";
import { findReceiptScreenElements } from "./find-receipt-screen-elements";
import { formatAmountInput } from "./format-amount-input";
import { frenchDateToIsoDate } from "./french-date-to-iso-date";
import {
  type ReceiptFormErrors,
  type ReceiptFormResult,
  type ReceiptFormValues,
  validateReceiptForm,
} from "./receipt-form";
import type { ReceiptGateway } from "./receipt-gateway";
import type { ReceiptScreenModel, ValidatedAction } from "./receipt-screen-model";
import { readReceiptFormValues } from "./read-receipt-form-values";
import { renderReceiptScreen } from "./render-receipt-screen";
import {
  INITIAL_SCREEN_STATE,
  nextScreenState,
  type PreviewRequest,
  type ScreenEvent,
} from "./screen-state";
import { validateTenantProfileForm } from "./tenant-profile-form";
import type { TenantProfileGateway } from "./tenant-profile-gateway";
import type { TenantProfileState } from "./tenant-profile-state";
import { toCommandErrorCode } from "./to-command-error-code";

const LOG_CODES = {
  TEMPLATE_LISTING_FAILED: "UI_TEMPLATE_LISTING_FAILED",
  NO_TEMPLATE: "UI_NO_TEMPLATE",
  PREVIEW_FAILED: "UI_PREVIEW_FAILED",
  SEND_FAILED: "UI_SEND_FAILED",
  TENANT_PROFILE_LISTING_FAILED: "UI_TENANT_PROFILE_LISTING_FAILED",
  TENANT_PROFILE_LOADING_FAILED: "UI_TENANT_PROFILE_LOADING_FAILED",
  TENANT_PROFILE_UNKNOWN_TEMPLATE: "UI_TENANT_PROFILE_UNKNOWN_TEMPLATE",
  TENANT_PROFILE_SAVING_FAILED: "UI_TENANT_PROFILE_SAVING_FAILED",
} as const;

const IDLE_TENANT_PROFILE_STATE: TenantProfileState = { status: "idle" };

/**
 * Branche l'écran de quittance sur le balisage de `index.html` présent dans `root`,
 * puis charge la liste des modèles et celle des fiches locataire. La promesse se résout
 * une fois ces listes affichées.
 */
export async function mountReceiptScreen(options: {
  readonly root: ParentNode;
  readonly gateway: ReceiptGateway;
  readonly tenantProfileGateway: TenantProfileGateway;
}): Promise<void> {
  const { gateway, tenantProfileGateway } = options;
  const elements = findReceiptScreenElements(options.root);
  let model: ReceiptScreenModel = {
    screenState: INITIAL_SCREEN_STATE,
    fieldErrors: null,
    validatedAction: "preview",
    isPreviewStale: false,
    templates: [],
    templateLoadError: null,
    tenantProfileState: IDLE_TENANT_PROFILE_STATE,
  };

  const readValues = (): ReceiptFormValues => readReceiptFormValues(elements.fieldInputs);
  const update = (changes: Partial<ReceiptScreenModel>): void => {
    model = { ...model, ...changes };
    renderReceiptScreen({ elements, model, formValues: readValues() });
  };
  const dispatch = (event: ScreenEvent): void => {
    update({ screenState: nextScreenState(model.screenState, event) });
  };
  const focusFirstInvalidField = (): void => {
    elements.form.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
  };

  const fieldErrorsOf = (action: ValidatedAction): ReceiptFormErrors => {
    const values = readValues();
    const validation =
      action === "preview" ? validateScreenForm(values) : validateTenantProfileForm(values);
    return validation.isValid ? {} : validation.errors;
  };

  // Une saisie clôt la question ou le message en cours sur les fiches, devenus caducs.
  const applyFormEdit = (): void => {
    const { status } = model.tenantProfileState;
    const isTenantProfileBusy = status === "loading" || status === "saving";
    update({
      screenState: nextScreenState(model.screenState, { type: "formEdited" }),
      isPreviewStale: model.screenState.status !== "idle" || model.isPreviewStale,
      fieldErrors: model.fieldErrors === null ? null : fieldErrorsOf(model.validatedAction),
      tenantProfileState: isTenantProfileBusy
        ? model.tenantProfileState
        : IDLE_TENANT_PROFILE_STATE,
    });
  };

  // Choisir une fiche ne modifie pas la saisie : seul son chargement la remplace.
  const handleEdit = (event: Event): void => {
    if (event.target !== elements.tenantProfileSelect) {
      applyFormEdit();
    }
  };

  const requestPreview = (): void => {
    if (!canRequestPreview(model)) {
      return;
    }
    const validation = validateScreenForm(readValues());
    if (!validation.isValid) {
      update({ fieldErrors: validation.errors, validatedAction: "preview", isPreviewStale: false });
      focusFirstInvalidField();
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
        console.error(`[${LOG_CODES.PREVIEW_FAILED}] preview command failed`, reason);
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
        console.error(`[${LOG_CODES.SEND_FAILED}] send command failed`, reason);
        dispatch({ type: "sendFailed", code: toCommandErrorCode(reason) });
      },
    );
  };

  const showTenantProfiles = (entries: readonly TenantProfileEntry[]): void => {
    const ownerDocument = elements.tenantProfileSelect.ownerDocument;
    elements.tenantProfileSelect.replaceChildren(
      elements.tenantProfileEmptyOption,
      ...entries.map((entry) =>
        createOption({ ownerDocument, value: entry.id, label: entry.label }),
      ),
    );
  };

  const fillTenantProfile = (profile: TenantProfilePayload): void => {
    const { fieldInputs } = elements;
    fieldInputs.tenantName.value = profile.tenantName;
    fieldInputs.tenantAddress.value = profile.tenantAddress;
    fieldInputs.tenantEmail.value = profile.tenantEmail;
    fieldInputs.propertyAddress.value = profile.propertyAddress;
    fieldInputs.rentAmount.value = formatAmountInput(profile.rentCents);
    fieldInputs.chargesAmount.value = formatAmountInput(profile.chargesCents);
    elements.templateSelect.value = profile.templateId;
    model = { ...model, tenantProfileState: IDLE_TENANT_PROFILE_STATE };
    applyFormEdit();
  };

  // Une fiche inutilisable ne doit laisser aucune saisie à moitié remplie derrière elle.
  const resetFormAfterLoadingFailure = (code: CommandErrorCode): void => {
    Object.values(elements.fieldInputs).forEach((input) => {
      input.value = "";
    });
    elements.templateSelect.value = model.templates[0]?.id ?? "";
    elements.tenantProfileSelect.value = "";
    update({
      screenState: nextScreenState(model.screenState, { type: "formEdited" }),
      fieldErrors: null,
      validatedAction: "preview",
      isPreviewStale: false,
      tenantProfileState: { status: "failed", code },
    });
  };

  const isLoadingTenantProfile = (id: string): boolean => {
    const state = model.tenantProfileState;
    return state.status === "loading" && state.id === id;
  };

  const loadTenantProfile = (): void => {
    const id = elements.tenantProfileSelect.value;
    if (id === "") {
      update({ tenantProfileState: IDLE_TENANT_PROFILE_STATE });
      return;
    }
    update({ tenantProfileState: { status: "loading", id } });
    tenantProfileGateway.loadTenantProfile(id).then(
      (profile: TenantProfilePayload) => {
        if (!isLoadingTenantProfile(id)) {
          return;
        }
        if (!model.templates.some((template) => template.id === profile.templateId)) {
          console.error(
            `[${LOG_CODES.TENANT_PROFILE_UNKNOWN_TEMPLATE}] the loaded tenant profile names the unlisted template ${profile.templateId}`,
          );
          resetFormAfterLoadingFailure("tenantProfile");
          return;
        }
        fillTenantProfile(profile);
      },
      (reason: unknown) => {
        if (!isLoadingTenantProfile(id)) {
          return;
        }
        console.error(
          `[${LOG_CODES.TENANT_PROFILE_LOADING_FAILED}] tenant profile loading command failed`,
          reason,
        );
        resetFormAfterLoadingFailure(toCommandErrorCode(reason));
      },
    );
  };

  const failTenantProfileCommand = (failure: {
    readonly logCode: string;
    readonly command: string;
    readonly reason: unknown;
  }): void => {
    console.error(`[${failure.logCode}] ${failure.command} command failed`, failure.reason);
    update({ tenantProfileState: { status: "failed", code: toCommandErrorCode(failure.reason) } });
  };

  const showSavedTenantProfile = async (entry: TenantProfileEntry): Promise<void> => {
    try {
      showTenantProfiles(await tenantProfileGateway.listTenantProfiles());
    } catch (reason: unknown) {
      failTenantProfileCommand({
        logCode: LOG_CODES.TENANT_PROFILE_LISTING_FAILED,
        command: "tenant profile listing",
        reason,
      });
      return;
    }
    elements.tenantProfileSelect.value = entry.id;
    update({ tenantProfileState: { status: "saved", entry } });
  };

  const failSavingTenantProfile = (reason: unknown): void => {
    failTenantProfileCommand({
      logCode: LOG_CODES.TENANT_PROFILE_SAVING_FAILED,
      command: "tenant profile saving",
      reason,
    });
  };

  const saveTenantProfile = (): void => {
    if (!canSaveTenantProfile(model)) {
      return;
    }
    const validation = validateTenantProfileForm(readValues());
    if (!validation.isValid) {
      update({
        fieldErrors: validation.errors,
        validatedAction: "saveTenantProfile",
        tenantProfileState: IDLE_TENANT_PROFILE_STATE,
      });
      focusFirstInvalidField();
      return;
    }
    const profile: TenantProfilePayload = {
      templateId: elements.templateSelect.value,
      ...validation.fields,
    };
    update({
      fieldErrors: model.validatedAction === "saveTenantProfile" ? null : model.fieldErrors,
      tenantProfileState: { status: "saving" },
    });
    tenantProfileGateway.createTenantProfile(profile).then(
      (entry: TenantProfileEntry) => showSavedTenantProfile(entry),
      (reason: unknown) => {
        if (toCommandErrorCode(reason) === "tenantProfileExists") {
          update({ tenantProfileState: { status: "confirmingReplacement", profile } });
          return;
        }
        failSavingTenantProfile(reason);
      },
    );
  };

  const replaceTenantProfile = (): void => {
    const state = model.tenantProfileState;
    if (state.status !== "confirmingReplacement") {
      return;
    }
    update({ tenantProfileState: { status: "saving" } });
    tenantProfileGateway
      .replaceTenantProfile(state.profile)
      .then((entry: TenantProfileEntry) => showSavedTenantProfile(entry), failSavingTenantProfile);
  };

  const cancelTenantProfileReplacement = (): void => {
    if (model.tenantProfileState.status === "confirmingReplacement") {
      update({ tenantProfileState: IDLE_TENANT_PROFILE_STATE });
    }
  };

  const listTemplates = async (): Promise<void> => {
    try {
      const templates = await gateway.listTemplates();
      if (templates.length === 0) {
        console.error(`[${LOG_CODES.NO_TEMPLATE}] the backend lists no receipt template`);
        update({ templateLoadError: "template" });
        return;
      }
      const ownerDocument = elements.templateSelect.ownerDocument;
      elements.templateSelect.replaceChildren(
        ...templates.map((template) =>
          createOption({ ownerDocument, value: template.id, label: template.label }),
        ),
      );
      update({ templates });
    } catch (reason: unknown) {
      console.error(
        `[${LOG_CODES.TEMPLATE_LISTING_FAILED}] template listing command failed`,
        reason,
      );
      update({ templateLoadError: toCommandErrorCode(reason) });
    }
  };

  const listTenantProfiles = async (): Promise<void> => {
    try {
      showTenantProfiles(await tenantProfileGateway.listTenantProfiles());
    } catch (reason: unknown) {
      failTenantProfileCommand({
        logCode: LOG_CODES.TENANT_PROFILE_LISTING_FAILED,
        command: "tenant profile listing",
        reason,
      });
    }
  };

  // `input` couvre aussi la liste des modèles : un changement de modèle invalide l'aperçu.
  elements.form.addEventListener("input", handleEdit);
  elements.form.addEventListener("submit", (event: SubmitEvent) => {
    event.preventDefault();
    requestPreview();
  });
  elements.sendButton.addEventListener("click", requestSend);
  elements.retryButton.addEventListener("click", requestSend);
  elements.tenantProfileSelect.addEventListener("change", loadTenantProfile);
  elements.saveTenantProfileButton.addEventListener("click", saveTenantProfile);
  elements.replaceTenantProfileButton.addEventListener("click", replaceTenantProfile);
  elements.cancelReplaceTenantProfileButton.addEventListener(
    "click",
    cancelTenantProfileReplacement,
  );
  update({});

  await Promise.all([listTemplates(), listTenantProfiles()]);
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
