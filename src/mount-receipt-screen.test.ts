// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import screenHtml from "../index.html?raw";
import type {
  CommandErrorCode,
  ReceiptInput,
  TemplateInfo,
  TenantProfileEntry,
  TenantProfilePayload,
} from "./api";
import { COMMAND_ERROR_MESSAGES } from "./command-error-messages";
import { FIELD_ERROR_MESSAGES } from "./field-error-messages";
import { mountReceiptScreen } from "./mount-receipt-screen";
import type { FieldErrorCode, ReceiptFormField, ReceiptFormValues } from "./receipt-form";
import type { ReceiptGateway } from "./receipt-gateway";
import type { PreviewRequest } from "./screen-state";
import { TENANT_PROFILE_FAILURE_MESSAGES } from "./tenant-profile-failure-messages";
import type { TenantProfileGateway } from "./tenant-profile-gateway";

// Identifiants stables du balisage, partagés avec les futurs tests e2e.
const FIELD_IDS: Readonly<Record<ReceiptFormField, string>> = {
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

const TEMPLATES: readonly TemplateInfo[] = [
  { id: "standard", label: "Standard" },
  { id: "compact", label: "Compact" },
];

const TENANT_PROFILES: readonly TenantProfileEntry[] = [
  { id: "jeanne-martin", label: "Jeanne Martin" },
  { id: "leo-durand", label: "Léo Durand" },
];

const JEANNE_PROFILE: TenantProfilePayload = {
  templateId: "compact",
  tenantName: "Jeanne Martin",
  tenantAddress: "8 rue Victor Hugo\n75011 Paris",
  tenantEmail: "jeanne.martin@example.fr",
  propertyAddress: "12 rue des Lilas\n75011 Paris",
  rentCents: 65_000,
  chargesCents: 5_050,
};

const LEO_PROFILE: TenantProfilePayload = {
  templateId: "standard",
  tenantName: "Léo Durand",
  tenantAddress: "3 place Bellecour\n69002 Lyon",
  tenantEmail: "leo.durand@example.fr",
  propertyAddress: "5 rue Mercière\n69002 Lyon",
  rentCents: 72_000,
  chargesCents: 9_000,
};

// Saisie de la maquette (état « Aperçu chargé »), dates au format jj/mm/aaaa.
const FILLED_VALUES: ReceiptFormValues = {
  tenantName: "Marie Dupont",
  tenantAddress: "12 rue des Lilas, appt 4B\n69003 Lyon",
  tenantEmail: "marie.dupont@exemple.fr",
  propertyAddress: "12 rue des Lilas, appt 4B\n69003 Lyon",
  periodStart: "01/09/2026",
  periodEnd: "30/09/2026",
  rentAmount: "650",
  chargesAmount: "80",
  paymentDate: "03/09/2026",
};

const FILLED_INPUT: ReceiptInput = {
  tenantName: "Marie Dupont",
  tenantAddress: "12 rue des Lilas, appt 4B\n69003 Lyon",
  tenantEmail: "marie.dupont@exemple.fr",
  propertyAddress: "12 rue des Lilas, appt 4B\n69003 Lyon",
  periodStart: "2026-09-01",
  periodEnd: "2026-09-30",
  rentCents: 65_000,
  chargesCents: 8_000,
  paymentDate: "2026-09-03",
};

// Saisie erronée de la maquette (état « Erreurs (a) ») et les erreurs attendues.
const INVALID_VALUES: ReceiptFormValues = {
  tenantName: "",
  tenantAddress: "12 rue des Lilas, appt 4B\n69003 Lyon",
  tenantEmail: "marie.dupont@exemple",
  propertyAddress: "",
  periodStart: "01/09/2026",
  periodEnd: "31/08/2026",
  rentAmount: "1 234,567",
  chargesAmount: "-20",
  paymentDate: "31/09/2026",
};

const INVALID_VALUES_ERRORS: Readonly<Partial<Record<ReceiptFormField, FieldErrorCode>>> = {
  tenantName: "missing",
  tenantEmail: "invalidEmail",
  propertyAddress: "missing",
  periodEnd: "invertedPeriod",
  rentAmount: "tooManyDecimals",
  chargesAmount: "negativeAmount",
  paymentDate: "invalidDate",
};

const PREVIEW_HTML = `<!doctype html>
<html lang="fr"><head><meta charset="utf-8"><title>Quittance de loyer</title></head>
<body><h1>Quittance de loyer</h1><p>Période du 1er septembre 2026 au 30 septembre 2026</p>
<p>Total reçu : 730,00 €</p></body></html>`;

const COMMAND_ERROR_CODES: readonly CommandErrorCode[] = [
  "validation",
  "template",
  "signature",
  "pdf",
  "mail",
  "config",
  "unknown",
];

// Tout log d'erreur commence par son code entre crochets.
const LOGGED_ERROR_CODE = /^\[UI_[A-Z_]+\] /;

interface PendingCall<T> {
  readonly request: PreviewRequest;
  readonly resolve: (value: T) => void;
  readonly reject: (reason: unknown) => void;
}

interface FakeGateway {
  readonly gateway: ReceiptGateway;
  readonly previewCalls: PendingCall<string>[];
  readonly sendCalls: PendingCall<void>[];
  readonly tenantProfiles: FakeTenantProfileGateway;
}

interface PendingTenantProfileCall<Argument, Result> {
  readonly argument: Argument;
  readonly resolve: (value: Result) => void;
  readonly reject: (reason: unknown) => void;
}

interface FakeTenantProfileGateway {
  readonly gateway: TenantProfileGateway;
  /** Répond la liste `TENANT_PROFILES` tant qu'un test ne la remplace pas. */
  readonly list: ReturnType<typeof vi.fn<TenantProfileGateway["listTenantProfiles"]>>;
  readonly loadCalls: PendingTenantProfileCall<string, TenantProfilePayload>[];
  readonly createCalls: PendingTenantProfileCall<TenantProfilePayload, TenantProfileEntry>[];
  readonly replaceCalls: PendingTenantProfileCall<TenantProfilePayload, TenantProfileEntry>[];
}

// Fake du backend : chaque commande reste en attente jusqu'à ce que le test la résolve.
function createFakeGateway(templates: Promise<readonly TemplateInfo[]>): FakeGateway {
  const previewCalls: PendingCall<string>[] = [];
  const sendCalls: PendingCall<void>[] = [];
  const gateway: ReceiptGateway = {
    listTemplates: () => templates,
    renderPreview: (request: PreviewRequest) =>
      new Promise<string>((resolve, reject) => {
        previewCalls.push({ request, resolve, reject });
      }),
    sendReceipt: (request: PreviewRequest) =>
      new Promise<void>((resolve, reject) => {
        sendCalls.push({ request, resolve, reject });
      }),
  };
  return { gateway, previewCalls, sendCalls, tenantProfiles: createFakeTenantProfileGateway() };
}

// Fake des commandes de fiches : seule la liste répond d'elle-même, les autres attendent le test.
function createFakeTenantProfileGateway(): FakeTenantProfileGateway {
  const list = vi
    .fn<TenantProfileGateway["listTenantProfiles"]>()
    .mockResolvedValue(TENANT_PROFILES);
  const loadCalls: PendingTenantProfileCall<string, TenantProfilePayload>[] = [];
  const createCalls: PendingTenantProfileCall<TenantProfilePayload, TenantProfileEntry>[] = [];
  const replaceCalls: PendingTenantProfileCall<TenantProfilePayload, TenantProfileEntry>[] = [];
  const pending =
    <Argument, Result>(calls: PendingTenantProfileCall<Argument, Result>[]) =>
    (argument: Argument): Promise<Result> =>
      new Promise<Result>((resolve, reject) => {
        calls.push({ argument, resolve, reject });
      });
  const gateway: TenantProfileGateway = {
    listTenantProfiles: list,
    loadTenantProfile: pending(loadCalls),
    createTenantProfile: pending(createCalls),
    replaceTenantProfile: pending(replaceCalls),
  };
  return { gateway, list, loadCalls, createCalls, replaceCalls };
}

function mountFake(fake: FakeGateway): Promise<void> {
  return mountReceiptScreen({
    root: document,
    gateway: fake.gateway,
    tenantProfileGateway: fake.tenantProfiles.gateway,
  });
}

async function mountScreen(
  templates: Promise<readonly TemplateInfo[]> = Promise.resolve(TEMPLATES),
): Promise<FakeGateway> {
  const fake = createFakeGateway(templates);
  await mountFake(fake);
  return fake;
}

function settle(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function byId<T extends HTMLElement = HTMLElement>(id: string): T {
  const element = document.getElementById(id);
  if (element === null) {
    throw new Error(`missing element #${id}`);
  }
  return element as T;
}

function fieldInput(field: ReceiptFormField): HTMLInputElement | HTMLTextAreaElement {
  return byId<HTMLInputElement | HTMLTextAreaElement>(FIELD_IDS[field]);
}

function typeInto(field: ReceiptFormField, value: string): void {
  const input = fieldInput(field);
  input.value = value;
  input.dispatchEvent(new Event("input", { bubbles: true }));
}

function fillForm(values: ReceiptFormValues): void {
  (Object.keys(values) as ReceiptFormField[]).forEach((field) => typeInto(field, values[field]));
}

function chooseTemplate(templateId: string): void {
  const select = byId<HTMLSelectElement>("template");
  select.value = templateId;
  select.dispatchEvent(new Event("input", { bubbles: true }));
  select.dispatchEvent(new Event("change", { bubbles: true }));
}

async function clickButton(id: string): Promise<void> {
  byId<HTMLButtonElement>(id).click();
  await settle();
}

async function previewFilledReceipt(fake: FakeGateway): Promise<void> {
  fillForm(FILLED_VALUES);
  await clickButton("preview-button");
  fake.previewCalls.at(-1)?.resolve(PREVIEW_HTML);
  await settle();
}

async function failSending(fake: FakeGateway, code: CommandErrorCode): Promise<void> {
  await previewFilledReceipt(fake);
  await clickButton("send-button");
  fake.sendCalls.at(-1)?.reject({ code, message: "détail technique" });
  await settle();
}

function isShown(id: string): boolean {
  return !byId(id).hidden;
}

function statusText(): string {
  return byId("preview-status").textContent?.trim() ?? "";
}

function plainText(id: string): string {
  return (byId(id).textContent ?? "").replace(/\s+/g, " ").trim();
}

function isLocked(element: HTMLElement): boolean {
  return element.closest("fieldset[disabled]") !== null;
}

function fieldErrorText(field: ReceiptFormField): string {
  const describedBy = fieldInput(field).getAttribute("aria-describedby");
  return describedBy === null ? "" : plainText(describedBy);
}

beforeEach(() => {
  document.body.innerHTML = new DOMParser().parseFromString(screenHtml, "text/html").body.innerHTML;
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("mountReceiptScreen: empty screen", () => {
  it("lists the backend templates in the template selector", async () => {
    // Arrange & Act
    await mountScreen();

    // Assert
    const options = Array.from(byId<HTMLSelectElement>("template").options).map((option) => ({
      id: option.value,
      label: option.textContent,
    }));
    expect(options).toEqual(TEMPLATES);
  });

  it("invites to fill the form while sending is disabled", async () => {
    // Arrange & Act
    await mountScreen();

    // Assert
    expect(isShown("preview-empty")).toBe(true);
    expect(isShown("preview-loading")).toBe(false);
    expect(isShown("preview-failure")).toBe(false);
    expect(isShown("preview-page")).toBe(false);
    expect(statusText()).toBe("Non généré");
    expect(byId<HTMLButtonElement>("preview-button").disabled).toBe(false);
    expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
    expect(isShown("retry-button")).toBe(false);
  });

  it("shows the total of rent and charges as the user types", async () => {
    // Arrange
    await mountScreen();

    // Act
    typeInto("rentAmount", "650");
    typeInto("chargesAmount", "80");

    // Assert
    expect(plainText("total")).toBe("730,00 €");
  });

  it("shows a dash as total while an amount is invalid", async () => {
    // Arrange
    await mountScreen();

    // Act
    typeInto("rentAmount", "six cent cinquante");
    typeInto("chargesAmount", "80");

    // Assert
    expect(plainText("total")).toBe("—");
  });

  it("reports a template listing failure and keeps the preview disabled", async () => {
    // Arrange
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const rejection = { code: "config", message: "LANDLORD_NAME manquant" };

    // Act
    await mountScreen(Promise.reject(rejection));

    // Assert
    expect(isShown("preview-failure")).toBe(true);
    expect(isShown("preview-empty")).toBe(false);
    expect(plainText("preview-failure")).toContain(COMMAND_ERROR_MESSAGES.config.title);
    expect(byId<HTMLButtonElement>("preview-button").disabled).toBe(true);
    expect(consoleError).toHaveBeenCalledWith(expect.any(String), rejection);
  });

  it("reports an empty template list as a template error", async () => {
    // Arrange
    vi.spyOn(console, "error").mockImplementation(() => undefined);

    // Act
    await mountScreen(Promise.resolve([]));

    // Assert
    expect(isShown("preview-failure")).toBe(true);
    expect(plainText("preview-failure")).toContain(COMMAND_ERROR_MESSAGES.template.title);
    expect(byId<HTMLButtonElement>("preview-button").disabled).toBe(true);
  });

  it("hides the hint to rerun the preview when templates cannot be listed", async () => {
    // Arrange
    vi.spyOn(console, "error").mockImplementation(() => undefined);

    // Act
    await mountScreen(Promise.reject({ code: "config" }));

    // Assert
    const hint = byId("preview-failure").querySelector<HTMLElement>(".pane-failure-hint");
    expect(hint?.hidden).toBe(true);
  });

  it("keeps the preview disabled until the templates are listed", async () => {
    // Arrange
    const fake = createFakeGateway(new Promise<readonly TemplateInfo[]>(() => undefined));

    // Act
    void mountFake(fake);
    await settle();

    // Assert
    expect(byId<HTMLButtonElement>("preview-button").disabled).toBe(true);
  });

  it("shows the application title bar", async () => {
    // Arrange & Act
    await mountScreen();

    // Assert
    expect(plainText("title-bar")).toBe("Quittances de loyer");
  });

  it("rejects with an explicit error when the markup lacks an expected element", async () => {
    // Arrange
    byId("preview-frame").remove();
    const fake = createFakeGateway(Promise.resolve(TEMPLATES));

    // Act
    const mounting = mountFake(fake);

    // Assert
    await expect(mounting).rejects.toThrow("UI_MARKUP_MISMATCH");
  });
});

describe("mountReceiptScreen: error logging", () => {
  it("logs a template listing failure with an error code", async () => {
    // Arrange
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const rejection = { code: "config" };

    // Act
    await mountScreen(Promise.reject(rejection));

    // Assert
    expect(consoleError).toHaveBeenCalledWith(expect.stringMatching(LOGGED_ERROR_CODE), rejection);
  });

  it("logs a preview failure with an error code", async () => {
    // Arrange
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    fillForm(FILLED_VALUES);
    await clickButton("preview-button");
    const rejection = { code: "template" };

    // Act
    fake.previewCalls[0]?.reject(rejection);
    await settle();

    // Assert
    expect(consoleError).toHaveBeenCalledWith(expect.stringMatching(LOGGED_ERROR_CODE), rejection);
  });

  it("logs a send failure with an error code", async () => {
    // Arrange
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();

    // Act
    await failSending(fake, "mail");

    // Assert
    expect(consoleError).toHaveBeenCalledWith(expect.stringMatching(LOGGED_ERROR_CODE), {
      code: "mail",
      message: "détail technique",
    });
  });
});

describe("mountReceiptScreen: input errors", () => {
  it("marks every invalid field and links it to its message", async () => {
    // Arrange
    await mountScreen();
    fillForm(INVALID_VALUES);

    // Act
    await clickButton("preview-button");

    // Assert
    (Object.keys(FIELD_IDS) as ReceiptFormField[]).forEach((field) => {
      const expectedError = INVALID_VALUES_ERRORS[field];
      const input = fieldInput(field);
      if (expectedError === undefined) {
        expect(input.getAttribute("aria-invalid"), field).toBe("false");
        expect(input.hasAttribute("aria-describedby"), field).toBe(false);
      } else {
        expect(input.getAttribute("aria-invalid"), field).toBe("true");
        expect(fieldErrorText(field), field).toContain(FIELD_ERROR_MESSAGES[expectedError]);
      }
    });
  });

  it.each<[ReceiptFormField, string, FieldErrorCode]>([
    ["rentAmount", "six cent cinquante", "invalidAmount"],
    ["chargesAmount", "90071992547409,92", "amountTooLarge"],
    ["periodStart", "le 1er septembre", "invalidDate"],
  ])("reports %s %j as %s", async (field, value, expectedError) => {
    // Arrange
    await mountScreen();
    fillForm({ ...FILLED_VALUES, [field]: value });

    // Act
    await clickButton("preview-button");

    // Assert
    expect(fieldInput(field).getAttribute("aria-invalid")).toBe("true");
    expect(fieldErrorText(field)).toContain(FIELD_ERROR_MESSAGES[expectedError]);
  });

  it("summarizes the number of fields to correct", async () => {
    // Arrange
    await mountScreen();
    fillForm(INVALID_VALUES);

    // Act
    await clickButton("preview-button");

    // Assert
    expect(isShown("error-summary")).toBe(true);
    expect(byId("error-summary").getAttribute("role")).toBe("alert");
    expect(plainText("error-summary")).toContain("7");
  });

  it("does not request a preview while the form is invalid", async () => {
    // Arrange
    const fake = await mountScreen();
    fillForm(INVALID_VALUES);

    // Act
    await clickButton("preview-button");

    // Assert
    expect(fake.previewCalls).toEqual([]);
    expect(statusText()).toBe("Non généré");
    expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
  });

  it("focuses the first invalid field", async () => {
    // Arrange
    await mountScreen();
    fillForm({ ...FILLED_VALUES, tenantEmail: "marie", rentAmount: "" });

    // Act
    await clickButton("preview-button");

    // Assert
    expect(document.activeElement).toBe(fieldInput("tenantEmail"));
  });

  it("does not show errors before the first preview attempt", async () => {
    // Arrange
    await mountScreen();

    // Act
    typeInto("tenantEmail", "marie");

    // Assert
    expect(fieldInput("tenantEmail").getAttribute("aria-invalid")).not.toBe("true");
    expect(isShown("error-summary")).toBe(false);
  });

  it("updates the errors as the user corrects a field after a failed attempt", async () => {
    // Arrange
    await mountScreen();
    fillForm(INVALID_VALUES);
    await clickButton("preview-button");

    // Act
    typeInto("tenantName", "Marie Dupont");

    // Assert
    expect(fieldInput("tenantName").getAttribute("aria-invalid")).toBe("false");
    expect(fieldInput("tenantName").hasAttribute("aria-describedby")).toBe(false);
    expect(plainText("error-summary")).toContain("6");
  });
});

describe("mountReceiptScreen: preview pending", () => {
  it("requests the preview of the validated input with the selected template", async () => {
    // Arrange
    const fake = await mountScreen();
    fillForm(FILLED_VALUES);

    // Act
    await clickButton("preview-button");

    // Assert
    expect(fake.previewCalls.map((call) => call.request)).toEqual([
      { input: FILLED_INPUT, templateId: "standard" },
    ]);
  });

  it("shows a loading indicator and blocks sending while the preview is generated", async () => {
    // Arrange
    await mountScreen();
    fillForm(FILLED_VALUES);

    // Act
    await clickButton("preview-button");

    // Assert
    expect(isShown("preview-loading")).toBe(true);
    expect(isShown("preview-empty")).toBe(false);
    expect(isShown("preview-page")).toBe(false);
    expect(byId("preview-pane").getAttribute("aria-busy")).toBe("true");
    expect(statusText()).toBe("Génération…");
    expect(byId<HTMLButtonElement>("preview-button").disabled).toBe(true);
    expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
    expect(plainText("announcement")).not.toBe("");
  });

  it("ignores a preview that arrives after the form was edited", async () => {
    // Arrange
    const fake = await mountScreen();
    fillForm(FILLED_VALUES);
    await clickButton("preview-button");
    typeInto("rentAmount", "700");

    // Act
    fake.previewCalls[0]?.resolve(PREVIEW_HTML);
    await settle();

    // Assert
    expect(isShown("preview-page")).toBe(false);
    expect(statusText()).toBe("À régénérer");
    expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
  });

  it("ignores a preview failure that arrives after the form was edited", async () => {
    // Arrange
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    fillForm(FILLED_VALUES);
    await clickButton("preview-button");
    typeInto("rentAmount", "700");

    // Act
    fake.previewCalls[0]?.reject({ code: "template" });
    await settle();

    // Assert
    expect(isShown("preview-failure")).toBe(false);
    expect(isShown("preview-empty")).toBe(true);
    expect(statusText()).toBe("À régénérer");
    expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
  });

  it.each([
    {
      outcome: "preview",
      answer: (call: PendingCall<string> | undefined) => call?.resolve(PREVIEW_HTML),
    },
    {
      outcome: "failure",
      answer: (call: PendingCall<string> | undefined) => call?.reject({ code: "template" }),
    },
  ])(
    "ignores the $outcome of a first request answering while a second one is pending",
    async ({ answer }) => {
      // Arrange
      vi.spyOn(console, "error").mockImplementation(() => undefined);
      const fake = await mountScreen();
      fillForm(FILLED_VALUES);
      await clickButton("preview-button");
      typeInto("rentAmount", "700");
      await clickButton("preview-button");

      // Act
      answer(fake.previewCalls[0]);
      await settle();

      // Assert
      expect(fake.previewCalls).toHaveLength(2);
      expect(statusText()).toBe("Génération…");
      expect(isShown("preview-loading")).toBe(true);
      expect(isShown("preview-page")).toBe(false);
      expect(isShown("preview-failure")).toBe(false);
      expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
    },
  );
});

describe("mountReceiptScreen: preview ready", () => {
  it("shows the recipient while the preview needs regenerating", async () => {
    // Arrange
    const fake = await mountScreen();
    await previewFilledReceipt(fake);

    // Act
    typeInto("chargesAmount", "90");

    // Assert
    expect(isShown("recipient-line")).toBe(true);
    expect(plainText("recipient-line")).toContain(FILLED_INPUT.tenantEmail);
  });

  it("shows the backend preview in a sandboxed frame and enables sending", async () => {
    // Arrange
    const fake = await mountScreen();

    // Act
    await previewFilledReceipt(fake);

    // Assert
    const frame = byId<HTMLIFrameElement>("preview-frame");
    expect(isShown("preview-page")).toBe(true);
    expect(isShown("preview-loading")).toBe(false);
    expect(frame.getAttribute("srcdoc")).toBe(PREVIEW_HTML);
    expect(frame.getAttribute("sandbox")).toBe("");
    expect(byId("preview-pane").getAttribute("aria-busy")).toBe("false");
    expect(statusText()).toBe("À jour");
    expect(byId<HTMLButtonElement>("send-button").disabled).toBe(false);
    expect(plainText("recipient-line")).toContain(FILLED_INPUT.tenantEmail);
  });

  it("invalidates the preview when a field is edited", async () => {
    // Arrange
    const fake = await mountScreen();
    await previewFilledReceipt(fake);

    // Act
    typeInto("chargesAmount", "90");

    // Assert
    expect(statusText()).toBe("À régénérer");
    expect(isShown("stale-banner")).toBe(true);
    expect(isShown("preview-page")).toBe(false);
    expect(isShown("preview-empty")).toBe(true);
    expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
    expect(fake.previewCalls).toHaveLength(1);
  });

  it("invalidates the preview when another template is chosen", async () => {
    // Arrange
    const fake = await mountScreen();
    await previewFilledReceipt(fake);

    // Act
    chooseTemplate("compact");

    // Assert
    expect(statusText()).toBe("À régénérer");
    expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
  });

  it("previews the newly chosen template", async () => {
    // Arrange
    const fake = await mountScreen();
    await previewFilledReceipt(fake);
    chooseTemplate("compact");

    // Act
    await clickButton("preview-button");

    // Assert
    expect(fake.previewCalls.at(-1)?.request).toEqual({
      input: FILLED_INPUT,
      templateId: "compact",
    });
  });

  it("says not generated once a new attempt fails validation after an invalidated preview", async () => {
    // Arrange
    const fake = await mountScreen();
    await previewFilledReceipt(fake);
    typeInto("tenantName", "");

    // Act
    await clickButton("preview-button");

    // Assert
    expect(statusText()).toBe("Non généré");
    expect(isShown("stale-banner")).toBe(false);
    expect(isShown("error-summary")).toBe(true);
  });
});

describe("mountReceiptScreen: preview failure", () => {
  it.each(COMMAND_ERROR_CODES)(
    "explains a %s preview failure in the preview pane",
    async (code) => {
      // Arrange
      const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
      const fake = await mountScreen();
      fillForm(FILLED_VALUES);
      await clickButton("preview-button");
      const rejection = { code, message: "détail technique" };

      // Act
      fake.previewCalls[0]?.reject(rejection);
      await settle();

      // Assert
      expect(isShown("preview-failure")).toBe(true);
      expect(byId("preview-failure").getAttribute("role")).toBe("alert");
      expect(plainText("preview-failure")).toContain(COMMAND_ERROR_MESSAGES[code].title);
      expect(plainText("preview-failure")).toContain(COMMAND_ERROR_MESSAGES[code].text);
      expect(isShown("preview-loading")).toBe(false);
      expect(statusText()).toBe("Échec de l’aperçu");
      expect(byId<HTMLButtonElement>("preview-button").disabled).toBe(false);
      expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
      expect(isShown("retry-button")).toBe(false);
      expect(consoleError).toHaveBeenCalledWith(expect.any(String), rejection);
    },
  );

  it("explains an unrecognized rejection as an unknown error", async () => {
    // Arrange
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    fillForm(FILLED_VALUES);
    await clickButton("preview-button");

    // Act
    fake.previewCalls[0]?.reject(new Error("invoke failed"));
    await settle();

    // Assert
    expect(plainText("preview-failure")).toContain(COMMAND_ERROR_MESSAGES.unknown.title);
  });
});

describe("mountReceiptScreen: sending", () => {
  it("sends the previewed input with its template", async () => {
    // Arrange
    const fake = await mountScreen();
    await previewFilledReceipt(fake);

    // Act
    await clickButton("send-button");

    // Assert
    expect(fake.sendCalls.map((call) => call.request)).toEqual([
      { input: FILLED_INPUT, templateId: "standard" },
    ]);
  });

  it("locks the whole form while sending", async () => {
    // Arrange
    const fake = await mountScreen();
    await previewFilledReceipt(fake);

    // Act
    await clickButton("send-button");

    // Assert
    (Object.keys(FIELD_IDS) as ReceiptFormField[]).forEach((field) => {
      expect(isLocked(fieldInput(field)), field).toBe(true);
    });
    expect(isLocked(byId("template"))).toBe(true);
    expect(byId("receipt-form").getAttribute("aria-busy")).toBe("true");
    expect(byId<HTMLButtonElement>("preview-button").disabled).toBe(true);
    expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
    expect(plainText("send-button")).toBe("Envoi en cours…");
    expect(statusText()).toBe("Envoi…");
  });

  it("keeps the preview displayed while sending", async () => {
    // Arrange
    const fake = await mountScreen();
    await previewFilledReceipt(fake);

    // Act
    await clickButton("send-button");

    // Assert
    expect(isShown("preview-page")).toBe(true);
  });

  it("keeps the outcome of a send when an input event fires while sending", async () => {
    // Arrange
    const fake = await mountScreen();
    await previewFilledReceipt(fake);
    await clickButton("send-button");
    typeInto("rentAmount", "700");

    // Act
    fake.sendCalls[0]?.resolve();
    await settle();

    // Assert
    expect(isShown("send-success")).toBe(true);
    expect(statusText()).toBe("Envoyée");
  });
});

describe("mountReceiptScreen: sent", () => {
  it("confirms the sending with the recipient email", async () => {
    // Arrange
    const fake = await mountScreen();
    await previewFilledReceipt(fake);
    await clickButton("send-button");

    // Act
    fake.sendCalls[0]?.resolve();
    await settle();

    // Assert
    expect(isShown("send-success")).toBe(true);
    expect(plainText("send-success")).toContain(FILLED_INPUT.tenantEmail);
    expect(plainText("announcement")).toContain(FILLED_INPUT.tenantEmail);
    expect(statusText()).toBe("Envoyée");
    expect(isLocked(fieldInput("tenantName"))).toBe(false);
    expect(byId("receipt-form").getAttribute("aria-busy")).toBe("false");
    expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
    expect(plainText("send-button")).toBe("Envoyer");
    expect(isShown("recipient-line")).toBe(false);
  });
});

describe("mountReceiptScreen: send failure", () => {
  it.each(COMMAND_ERROR_CODES)("explains a %s send failure and offers to retry", async (code) => {
    // Arrange
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();

    // Act
    await failSending(fake, code);

    // Assert
    expect(isShown("send-failure")).toBe(true);
    expect(byId("send-failure").getAttribute("role")).toBe("alert");
    expect(plainText("send-failure")).toContain(COMMAND_ERROR_MESSAGES[code].title);
    expect(plainText("send-failure")).toContain(COMMAND_ERROR_MESSAGES[code].text);
    expect(isShown("retry-button")).toBe(true);
    expect(isShown("send-button")).toBe(false);
    expect(isShown("preview-page")).toBe(true);
    expect(isShown("preview-failure")).toBe(false);
    expect(statusText()).toBe("À jour · envoi échoué");
    expect(isLocked(fieldInput("tenantName"))).toBe(false);
    expect(consoleError).toHaveBeenCalledWith(expect.any(String), {
      code,
      message: "détail technique",
    });
  });

  it("retries the failed send without a new preview", async () => {
    // Arrange
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    await failSending(fake, "mail");

    // Act
    await clickButton("retry-button");
    fake.sendCalls[1]?.resolve();
    await settle();

    // Assert
    expect(fake.previewCalls).toHaveLength(1);
    expect(fake.sendCalls.map((call) => call.request)).toEqual([
      { input: FILLED_INPUT, templateId: "standard" },
      { input: FILLED_INPUT, templateId: "standard" },
    ]);
    expect(isShown("send-success")).toBe(true);
    expect(isShown("retry-button")).toBe(false);
  });

  it("drops the retry when the form is edited after a failed send", async () => {
    // Arrange
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    await failSending(fake, "mail");

    // Act
    typeInto("paymentDate", "04/09/2026");

    // Assert
    expect(isShown("retry-button")).toBe(false);
    expect(isShown("send-failure")).toBe(false);
    expect(isShown("send-button")).toBe(true);
    expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
    expect(statusText()).toBe("À régénérer");
  });
});

// Fiche de Marie Dupont, enregistrée depuis la saisie `FILLED_VALUES` avec le modèle compact.
const MARIE_PROFILE: TenantProfilePayload = {
  templateId: "compact",
  tenantName: "Marie Dupont",
  tenantAddress: "12 rue des Lilas, appt 4B\n69003 Lyon",
  tenantEmail: "marie.dupont@exemple.fr",
  propertyAddress: "12 rue des Lilas, appt 4B\n69003 Lyon",
  rentCents: 65_000,
  chargesCents: 8_000,
};

const MARIE_ENTRY: TenantProfileEntry = { id: "marie-dupont", label: "Marie Dupont" };

const TENANT_PROFILE_FIELDS: readonly ReceiptFormField[] = [
  "tenantName",
  "tenantAddress",
  "tenantEmail",
  "propertyAddress",
  "rentAmount",
  "chargesAmount",
];

function tenantProfileSelect(): HTMLSelectElement {
  return byId<HTMLSelectElement>("tenant-profile");
}

function chooseTenantProfile(id: string): void {
  const select = tenantProfileSelect();
  select.value = id;
  select.dispatchEvent(new Event("input", { bubbles: true }));
  select.dispatchEvent(new Event("change", { bubbles: true }));
}

async function loadTenantProfile(
  fake: FakeGateway,
  choice: { readonly id: string; readonly profile: TenantProfilePayload },
): Promise<void> {
  chooseTenantProfile(choice.id);
  await settle();
  fake.tenantProfiles.loadCalls.at(-1)?.resolve(choice.profile);
  await settle();
}

async function failLoadingTenantProfile(fake: FakeGateway, rejection: unknown): Promise<void> {
  chooseTenantProfile("jeanne-martin");
  await settle();
  fake.tenantProfiles.loadCalls.at(-1)?.reject(rejection);
  await settle();
}

function formValues(): ReceiptFormValues {
  return Object.fromEntries(
    (Object.keys(FIELD_IDS) as ReceiptFormField[]).map((field) => [field, fieldInput(field).value]),
  ) as unknown as ReceiptFormValues;
}

// Saisie remplie avec le modèle compact, puis demande d'enregistrement de la fiche.
async function saveFilledTenantProfile(): Promise<void> {
  fillForm(FILLED_VALUES);
  chooseTemplate("compact");
  await clickButton("save-tenant-profile-button");
}

async function askReplacement(fake: FakeGateway): Promise<void> {
  await saveFilledTenantProfile();
  fake.tenantProfiles.createCalls[0]?.reject({ code: "tenantProfileExists", message: "existe" });
  await settle();
}

function isInvalid(field: ReceiptFormField): boolean {
  return fieldInput(field).getAttribute("aria-invalid") === "true";
}

describe("mountReceiptScreen: tenant profile list", () => {
  it("lists the saved tenant profiles after an empty option selected by default", async () => {
    // Arrange & Act
    await mountScreen();

    // Assert
    const options = Array.from(tenantProfileSelect().options);
    expect(options[0]?.value).toBe("");
    expect(tenantProfileSelect().value).toBe("");
    expect(
      options.slice(1).map((option) => ({ id: option.value, label: option.textContent })),
    ).toEqual(TENANT_PROFILES);
  });

  it("offers only the empty option when no tenant profile is saved", async () => {
    // Arrange
    const fake = createFakeGateway(Promise.resolve(TEMPLATES));
    fake.tenantProfiles.list.mockResolvedValue([]);

    // Act
    await mountFake(fake);

    // Assert
    expect(Array.from(tenantProfileSelect().options).map((option) => option.value)).toEqual([""]);
  });

  it("places the tenant profile list above the other fields", async () => {
    // Arrange & Act
    await mountScreen();

    // Assert
    const firstControl = byId("form-controls").querySelector("input, textarea, select");
    expect(firstControl).toBe(tenantProfileSelect());
  });

  it("reports a listing failure and offers only the empty option", async () => {
    // Arrange
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = createFakeGateway(Promise.resolve(TEMPLATES));
    const rejection = { code: "tenantProfile", message: "dossier fiches illisible" };
    fake.tenantProfiles.list.mockRejectedValue(rejection);

    // Act
    await mountFake(fake);

    // Assert
    expect(isShown("tenant-profile-failure")).toBe(true);
    expect(byId("tenant-profile-failure").getAttribute("role")).toBe("alert");
    expect(plainText("tenant-profile-failure")).toContain(
      COMMAND_ERROR_MESSAGES.tenantProfile.title,
    );
    expect(Array.from(tenantProfileSelect().options).map((option) => option.value)).toEqual([""]);
    expect(consoleError).toHaveBeenCalledWith(expect.stringMatching(LOGGED_ERROR_CODE), rejection);
  });

  it("shows no tenant profile message on a successful mount", async () => {
    // Arrange & Act
    await mountScreen();

    // Assert
    expect(isShown("tenant-profile-failure")).toBe(false);
    expect(isShown("tenant-profile-replace")).toBe(false);
    expect(isShown("tenant-profile-saved")).toBe(false);
  });

  it("disables the tenant profile controls until the templates are listed", async () => {
    // Arrange
    const fake = createFakeGateway(new Promise<readonly TemplateInfo[]>(() => undefined));

    // Act
    void mountFake(fake);
    await settle();

    // Assert
    expect(tenantProfileSelect().disabled).toBe(true);
    expect(byId<HTMLButtonElement>("save-tenant-profile-button").disabled).toBe(true);
  });

  it("locks the tenant profile controls while a receipt is being sent", async () => {
    // Arrange
    const fake = await mountScreen();
    await previewFilledReceipt(fake);

    // Act
    await clickButton("send-button");

    // Assert
    expect(isLocked(tenantProfileSelect())).toBe(true);
    expect(isLocked(byId("save-tenant-profile-button"))).toBe(true);
  });
});

describe("mountReceiptScreen: tenant profile loading", () => {
  it("loads the chosen tenant profile", async () => {
    // Arrange
    const fake = await mountScreen();

    // Act
    chooseTenantProfile("jeanne-martin");
    await settle();

    // Assert
    expect(fake.tenantProfiles.loadCalls.map((call) => call.argument)).toEqual(["jeanne-martin"]);
  });

  it("fills the tenant, the property, the amounts and the template from the profile", async () => {
    // Arrange
    const fake = await mountScreen();

    // Act
    await loadTenantProfile(fake, { id: "jeanne-martin", profile: JEANNE_PROFILE });

    // Assert
    expect(formValues()).toEqual({
      tenantName: "Jeanne Martin",
      tenantAddress: "8 rue Victor Hugo\n75011 Paris",
      tenantEmail: "jeanne.martin@example.fr",
      propertyAddress: "12 rue des Lilas\n75011 Paris",
      periodStart: "",
      periodEnd: "",
      rentAmount: "650,00",
      chargesAmount: "50,50",
      paymentDate: "",
    });
    expect(byId<HTMLSelectElement>("template").value).toBe("compact");
    expect(plainText("total")).toBe("700,50 €");
    expect(tenantProfileSelect().value).toBe("jeanne-martin");
  });

  it("keeps the period and the payment date already typed", async () => {
    // Arrange
    const fake = await mountScreen();
    fillForm(FILLED_VALUES);

    // Act
    await loadTenantProfile(fake, { id: "jeanne-martin", profile: JEANNE_PROFILE });

    // Assert
    expect(formValues()).toMatchObject({
      periodStart: FILLED_VALUES.periodStart,
      periodEnd: FILLED_VALUES.periodEnd,
      paymentDate: FILLED_VALUES.paymentDate,
    });
  });

  it("previews the loaded tenant profile once the period and payment date are typed", async () => {
    // Arrange
    const fake = await mountScreen();
    await loadTenantProfile(fake, { id: "jeanne-martin", profile: JEANNE_PROFILE });
    typeInto("periodStart", "01/09/2026");
    typeInto("periodEnd", "30/09/2026");
    typeInto("paymentDate", "03/09/2026");

    // Act
    await clickButton("preview-button");

    // Assert
    expect(fake.previewCalls.map((call) => call.request)).toEqual([
      {
        templateId: "compact",
        input: {
          tenantName: "Jeanne Martin",
          tenantAddress: "8 rue Victor Hugo\n75011 Paris",
          tenantEmail: "jeanne.martin@example.fr",
          propertyAddress: "12 rue des Lilas\n75011 Paris",
          periodStart: "2026-09-01",
          periodEnd: "2026-09-30",
          rentCents: 65_000,
          chargesCents: 5_050,
          paymentDate: "2026-09-03",
        },
      },
    ]);
  });

  it("invalidates the displayed preview", async () => {
    // Arrange
    const fake = await mountScreen();
    await previewFilledReceipt(fake);

    // Act
    await loadTenantProfile(fake, { id: "jeanne-martin", profile: JEANNE_PROFILE });

    // Assert
    expect(statusText()).toBe("À régénérer");
    expect(byId<HTMLButtonElement>("send-button").disabled).toBe(true);
  });

  it("leaves the form untouched when the empty option is chosen", async () => {
    // Arrange
    const fake = await mountScreen();
    await loadTenantProfile(fake, { id: "jeanne-martin", profile: JEANNE_PROFILE });

    // Act
    chooseTenantProfile("");
    await settle();

    // Assert
    expect(fake.tenantProfiles.loadCalls).toHaveLength(1);
    expect(fieldInput("tenantName").value).toBe("Jeanne Martin");
  });

  it("ignores a tenant profile that arrives after another one was chosen", async () => {
    // Arrange
    const fake = await mountScreen();
    chooseTenantProfile("jeanne-martin");
    await settle();
    chooseTenantProfile("leo-durand");
    await settle();

    // Act
    fake.tenantProfiles.loadCalls[1]?.resolve(LEO_PROFILE);
    await settle();
    fake.tenantProfiles.loadCalls[0]?.resolve(JEANNE_PROFILE);
    await settle();

    // Assert
    expect(fieldInput("tenantName").value).toBe("Léo Durand");
    expect(tenantProfileSelect().value).toBe("leo-durand");
  });

  it("reports a loading failure and logs it with an error code", async () => {
    // Arrange
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    const rejection = { code: "tenantProfile", message: "clé inconnue landlord_name" };

    // Act
    await failLoadingTenantProfile(fake, rejection);

    // Assert
    expect(isShown("tenant-profile-failure")).toBe(true);
    expect(plainText("tenant-profile-failure")).toContain(
      COMMAND_ERROR_MESSAGES.tenantProfile.title,
    );
    expect(plainText("tenant-profile-failure")).toContain(
      COMMAND_ERROR_MESSAGES.tenantProfile.text,
    );
    expect(consoleError).toHaveBeenCalledWith(expect.stringMatching(LOGGED_ERROR_CODE), rejection);
  });

  it("resets the fields, the template and the tenant profile list on a loading failure", async () => {
    // Arrange
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    fillForm(FILLED_VALUES);
    chooseTemplate("compact");

    // Act
    await failLoadingTenantProfile(fake, { code: "tenantProfile" });

    // Assert
    expect(Object.values(formValues())).toEqual(Object.keys(FIELD_IDS).map(() => ""));
    expect(byId<HTMLSelectElement>("template").value).toBe("standard");
    expect(tenantProfileSelect().value).toBe("");
    expect(plainText("total")).toBe("—");
  });

  it("brings the screen back to its initial state on a loading failure", async () => {
    // Arrange
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    await previewFilledReceipt(fake);
    fillForm(INVALID_VALUES);
    await clickButton("preview-button");

    // Act
    await failLoadingTenantProfile(fake, { code: "tenantProfile" });

    // Assert
    expect((Object.keys(FIELD_IDS) as ReceiptFormField[]).filter(isInvalid)).toEqual([]);
    expect(isShown("error-summary")).toBe(false);
    expect(isShown("preview-empty")).toBe(true);
    expect(statusText()).toBe("Non généré");
  });

  it("rejects a tenant profile whose template is not listed", async () => {
    // Arrange
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    fillForm(FILLED_VALUES);

    // Act
    await loadTenantProfile(fake, {
      id: "jeanne-martin",
      profile: { ...JEANNE_PROFILE, templateId: "retired" },
    });

    // Assert
    expect(isShown("tenant-profile-failure")).toBe(true);
    expect(plainText("tenant-profile-failure")).toContain(
      COMMAND_ERROR_MESSAGES.tenantProfile.title,
    );
    expect(Object.values(formValues())).toEqual(Object.keys(FIELD_IDS).map(() => ""));
    expect(tenantProfileSelect().value).toBe("");
    expect(consoleError).toHaveBeenCalledWith(expect.stringMatching(LOGGED_ERROR_CODE));
  });

  it("hides the loading failure once a tenant profile loads", async () => {
    // Arrange
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    await failLoadingTenantProfile(fake, { code: "tenantProfile" });

    // Act
    await loadTenantProfile(fake, { id: "leo-durand", profile: LEO_PROFILE });

    // Assert
    expect(isShown("tenant-profile-failure")).toBe(false);
  });
});

describe("mountReceiptScreen: tenant profile saving", () => {
  it("creates a tenant profile from the form fields and the selected template", async () => {
    // Arrange
    const fake = await mountScreen();

    // Act
    await saveFilledTenantProfile();

    // Assert
    expect(fake.tenantProfiles.createCalls.map((call) => call.argument)).toEqual([MARIE_PROFILE]);
    expect(fake.previewCalls).toEqual([]);
  });

  it("saves a tenant profile without a period or a payment date", async () => {
    // Arrange
    const fake = await mountScreen();
    fillForm({ ...FILLED_VALUES, periodStart: "", periodEnd: "", paymentDate: "" });
    chooseTemplate("compact");

    // Act
    await clickButton("save-tenant-profile-button");

    // Assert
    expect(fake.tenantProfiles.createCalls.map((call) => call.argument)).toEqual([MARIE_PROFILE]);
  });

  it("saves the fields as edited after loading a tenant profile", async () => {
    // Arrange
    const fake = await mountScreen();
    await loadTenantProfile(fake, { id: "jeanne-martin", profile: JEANNE_PROFILE });
    typeInto("tenantEmail", "jeanne@example.fr");

    // Act
    await clickButton("save-tenant-profile-button");

    // Assert
    expect(fake.tenantProfiles.createCalls.map((call) => call.argument)).toEqual([
      { ...JEANNE_PROFILE, tenantEmail: "jeanne@example.fr" },
    ]);
  });

  it("marks only the invalid tenant profile fields and does not save", async () => {
    // Arrange
    const fake = await mountScreen();
    fillForm(INVALID_VALUES);

    // Act
    await clickButton("save-tenant-profile-button");

    // Assert
    expect(fake.tenantProfiles.createCalls).toEqual([]);
    (Object.keys(FIELD_IDS) as ReceiptFormField[]).forEach((field) => {
      const expectedError = TENANT_PROFILE_FIELDS.includes(field)
        ? INVALID_VALUES_ERRORS[field]
        : undefined;
      if (expectedError === undefined) {
        expect(isInvalid(field), field).toBe(false);
      } else {
        expect(isInvalid(field), field).toBe(true);
        expect(fieldErrorText(field), field).toContain(FIELD_ERROR_MESSAGES[expectedError]);
      }
    });
    expect(isShown("error-summary")).toBe(true);
    expect(plainText("error-summary")).toContain("5");
  });

  it("focuses the first invalid tenant profile field", async () => {
    // Arrange
    await mountScreen();
    fillForm({ ...FILLED_VALUES, tenantEmail: "marie", periodStart: "" });

    // Act
    await clickButton("save-tenant-profile-button");

    // Assert
    expect(document.activeElement).toBe(fieldInput("tenantEmail"));
  });

  it("updates the tenant profile errors as the user corrects a field", async () => {
    // Arrange
    await mountScreen();
    fillForm(INVALID_VALUES);
    await clickButton("save-tenant-profile-button");

    // Act
    typeInto("tenantName", "Marie Dupont");

    // Assert
    expect(isInvalid("tenantName")).toBe(false);
    expect(isInvalid("periodEnd")).toBe(false);
    expect(plainText("error-summary")).toContain("4");
  });

  it("refreshes the tenant profile list and selects the saved profile", async () => {
    // Arrange
    const fake = await mountScreen();
    await saveFilledTenantProfile();
    fake.tenantProfiles.list.mockResolvedValue([...TENANT_PROFILES, MARIE_ENTRY]);

    // Act
    fake.tenantProfiles.createCalls[0]?.resolve(MARIE_ENTRY);
    await settle();

    // Assert
    expect(fake.tenantProfiles.list).toHaveBeenCalledTimes(2);
    expect(Array.from(tenantProfileSelect().options).map((option) => option.value)).toEqual([
      "",
      "jeanne-martin",
      "leo-durand",
      "marie-dupont",
    ]);
    expect(tenantProfileSelect().value).toBe("marie-dupont");
    expect(isShown("tenant-profile-saved")).toBe(true);
    expect(plainText("tenant-profile-saved")).toContain("Marie Dupont");
  });

  it("disables saving while a tenant profile is being saved", async () => {
    // Arrange
    const fake = await mountScreen();
    await saveFilledTenantProfile();

    // Act
    await clickButton("save-tenant-profile-button");

    // Assert
    expect(byId<HTMLButtonElement>("save-tenant-profile-button").disabled).toBe(true);
    expect(fake.tenantProfiles.createCalls).toHaveLength(1);
  });

  it("reports a saving failure, logs it and allows saving again", async () => {
    // Arrange
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    await saveFilledTenantProfile();
    const rejection = { code: "tenantProfile", message: "permission refusée" };

    // Act
    fake.tenantProfiles.createCalls[0]?.reject(rejection);
    await settle();

    // Assert
    expect(isShown("tenant-profile-failure")).toBe(true);
    expect(plainText("tenant-profile-failure")).toContain(
      COMMAND_ERROR_MESSAGES.tenantProfile.title,
    );
    expect(isShown("tenant-profile-saved")).toBe(false);
    expect(byId<HTMLButtonElement>("save-tenant-profile-button").disabled).toBe(false);
    expect(consoleError).toHaveBeenCalledWith(expect.stringMatching(LOGGED_ERROR_CODE), rejection);
  });

  it("explains a tenant profile rejected by the backend in terms of the profile", async () => {
    // Arrange
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    await saveFilledTenantProfile();
    const rejection = { code: "validation", message: "adresse du logement vide" };

    // Act
    fake.tenantProfiles.createCalls[0]?.reject(rejection);
    await settle();

    // Assert
    expect(isShown("tenant-profile-failure")).toBe(true);
    expect(plainText("tenant-profile-failure")).toContain(
      TENANT_PROFILE_FAILURE_MESSAGES.saveRejected.title,
    );
    expect(plainText("tenant-profile-failure")).toContain(
      TENANT_PROFILE_FAILURE_MESSAGES.saveRejected.text,
    );
    expect(plainText("tenant-profile-failure")).not.toContain(
      COMMAND_ERROR_MESSAGES.validation.title,
    );
    expect(isShown("tenant-profile-saved")).toBe(false);
    expect(consoleError).toHaveBeenCalledWith(expect.stringMatching(LOGGED_ERROR_CODE), rejection);
  });

  it("tells that the tenant profile is saved when the list cannot be reloaded", async () => {
    // Arrange
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    await saveFilledTenantProfile();
    const rejection = { code: "tenantProfile", message: "dossier fiches illisible" };
    fake.tenantProfiles.list.mockRejectedValue(rejection);

    // Act
    fake.tenantProfiles.createCalls[0]?.resolve(MARIE_ENTRY);
    await settle();

    // Assert
    expect(isShown("tenant-profile-failure")).toBe(true);
    expect(plainText("tenant-profile-failure")).toContain(
      TENANT_PROFILE_FAILURE_MESSAGES.listNotRefreshed.title,
    );
    expect(plainText("tenant-profile-failure")).toContain(
      TENANT_PROFILE_FAILURE_MESSAGES.listNotRefreshed.text,
    );
    expect(plainText("tenant-profile-failure")).not.toContain(
      COMMAND_ERROR_MESSAGES.tenantProfile.title,
    );
    expect(byId<HTMLButtonElement>("save-tenant-profile-button").disabled).toBe(false);
    expect(consoleError).toHaveBeenCalledWith(expect.stringMatching(LOGGED_ERROR_CODE), rejection);
  });

  it("gives the focus back to the save button once the tenant profile is saved", async () => {
    // Arrange
    const fake = await mountScreen();
    await saveFilledTenantProfile();

    // Act
    fake.tenantProfiles.createCalls[0]?.resolve(MARIE_ENTRY);
    await settle();

    // Assert
    expect(document.activeElement).toBe(byId("save-tenant-profile-button"));
  });

  it("leaves the focus in the field the user moved to while saving", async () => {
    // Arrange
    const fake = await mountScreen();
    await saveFilledTenantProfile();
    fieldInput("periodStart").focus();

    // Act
    fake.tenantProfiles.createCalls[0]?.resolve(MARIE_ENTRY);
    await settle();

    // Assert
    expect(document.activeElement).toBe(fieldInput("periodStart"));
  });

  it("gives the focus back to the save button once saving fails", async () => {
    // Arrange
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    await saveFilledTenantProfile();

    // Act
    fake.tenantProfiles.createCalls[0]?.reject({ code: "tenantProfile" });
    await settle();

    // Assert
    expect(document.activeElement).toBe(byId("save-tenant-profile-button"));
  });

  it("hides the tenant profile messages once the form is edited", async () => {
    // Arrange
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    await saveFilledTenantProfile();
    fake.tenantProfiles.createCalls[0]?.reject({ code: "tenantProfile" });
    await settle();

    // Act
    typeInto("tenantName", "Marie Durand");

    // Assert
    expect(isShown("tenant-profile-failure")).toBe(false);
  });
});

describe("mountReceiptScreen: tenant profile replacement", () => {
  it("asks in the page before replacing an existing tenant profile", async () => {
    // Arrange
    const fake = await mountScreen();

    // Act
    await askReplacement(fake);

    // Assert
    expect(isShown("tenant-profile-replace")).toBe(true);
    expect(plainText("tenant-profile-replace")).toContain("Marie Dupont");
    expect(isShown("replace-tenant-profile-button")).toBe(true);
    expect(isShown("cancel-replace-tenant-profile-button")).toBe(true);
    expect(isShown("tenant-profile-failure")).toBe(false);
    expect(fake.tenantProfiles.replaceCalls).toEqual([]);
  });

  it("focuses the replace button when the replacement question shows", async () => {
    // Arrange
    const fake = await mountScreen();

    // Act
    await askReplacement(fake);

    // Assert
    expect(document.activeElement).toBe(byId("replace-tenant-profile-button"));
  });

  it("gives the focus back to the save button when the user cancels", async () => {
    // Arrange
    const fake = await mountScreen();
    await askReplacement(fake);

    // Act
    await clickButton("cancel-replace-tenant-profile-button");

    // Assert
    expect(document.activeElement).toBe(byId("save-tenant-profile-button"));
  });

  it("leaves the focus in the field the user moved to while replacing", async () => {
    // Arrange
    const fake = await mountScreen();
    await askReplacement(fake);
    await clickButton("replace-tenant-profile-button");
    fieldInput("paymentDate").focus();

    // Act
    fake.tenantProfiles.replaceCalls[0]?.resolve(MARIE_ENTRY);
    await settle();

    // Assert
    expect(document.activeElement).toBe(fieldInput("paymentDate"));
  });

  it("gives the focus back to the save button once the replacement is saved", async () => {
    // Arrange
    const fake = await mountScreen();
    await askReplacement(fake);
    await clickButton("replace-tenant-profile-button");

    // Act
    fake.tenantProfiles.replaceCalls[0]?.resolve(MARIE_ENTRY);
    await settle();

    // Assert
    expect(document.activeElement).toBe(byId("save-tenant-profile-button"));
  });

  it("replaces the tenant profile once the user confirms", async () => {
    // Arrange
    const fake = await mountScreen();
    await askReplacement(fake);

    // Act
    await clickButton("replace-tenant-profile-button");

    // Assert
    expect(fake.tenantProfiles.replaceCalls.map((call) => call.argument)).toEqual([MARIE_PROFILE]);
    expect(isShown("tenant-profile-replace")).toBe(false);
  });

  it("refreshes the list and selects the replaced tenant profile", async () => {
    // Arrange
    const fake = await mountScreen();
    await askReplacement(fake);
    await clickButton("replace-tenant-profile-button");
    fake.tenantProfiles.list.mockResolvedValue([...TENANT_PROFILES, MARIE_ENTRY]);

    // Act
    fake.tenantProfiles.replaceCalls[0]?.resolve(MARIE_ENTRY);
    await settle();

    // Assert
    expect(tenantProfileSelect().value).toBe("marie-dupont");
    expect(isShown("tenant-profile-saved")).toBe(true);
  });

  it("keeps the existing tenant profile when the user cancels", async () => {
    // Arrange
    const fake = await mountScreen();
    await askReplacement(fake);

    // Act
    await clickButton("cancel-replace-tenant-profile-button");

    // Assert
    expect(isShown("tenant-profile-replace")).toBe(false);
    expect(fake.tenantProfiles.replaceCalls).toEqual([]);
    expect(fake.tenantProfiles.createCalls).toHaveLength(1);
    expect(byId<HTMLButtonElement>("save-tenant-profile-button").disabled).toBe(false);
  });

  it("drops the replacement question when the form is edited", async () => {
    // Arrange
    const fake = await mountScreen();
    await askReplacement(fake);

    // Act
    typeInto("rentAmount", "700");

    // Assert
    expect(isShown("tenant-profile-replace")).toBe(false);
    expect(fake.tenantProfiles.replaceCalls).toEqual([]);
  });

  it("reports a replacement failure and logs it with an error code", async () => {
    // Arrange
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const fake = await mountScreen();
    await askReplacement(fake);
    await clickButton("replace-tenant-profile-button");
    const rejection = { code: "tenantProfile", message: "disque plein" };

    // Act
    fake.tenantProfiles.replaceCalls[0]?.reject(rejection);
    await settle();

    // Assert
    expect(isShown("tenant-profile-failure")).toBe(true);
    expect(plainText("tenant-profile-failure")).toContain(
      COMMAND_ERROR_MESSAGES.tenantProfile.title,
    );
    expect(consoleError).toHaveBeenCalledWith(expect.stringMatching(LOGGED_ERROR_CODE), rejection);
  });
});
