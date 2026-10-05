import { afterEach, describe, expect, it, vi } from "vitest";

import type { TemplateInfo } from "./api";
import type { InvokeCommand } from "./invoke-command";
import type { ReceiptGateway } from "./receipt-gateway";
import type { PreviewRequest } from "./screen-state";
import { createTauriReceiptGateway } from "./tauri-receipt-gateway";
import { sampleReceiptInput } from "./test-fixtures";
import { toCommandErrorCode } from "./to-command-error-code";

const TEMPLATES: readonly TemplateInfo[] = [
  { id: "standard", label: "Quittance standard" },
  { id: "compact", label: "Quittance compacte" },
];

const PREVIEW_HTML = "<!doctype html><html><body><h1>Quittance de loyer</h1></body></html>";

// Code de log porté par l'erreur d'une réponse du backend de forme inattendue.
const INVALID_RESPONSE_ERROR_CODE = "[UI_INVALID_COMMAND_RESPONSE]";

// Midi local : la date du jour ne dépend pas du fuseau de la machine de test.
const LOCAL_NOON = new Date(2026, 9, 5, 12, 0);

function previewRequest(): PreviewRequest {
  return { input: sampleReceiptInput(), templateId: "standard" };
}

function createGateway(options: {
  readonly response: () => Promise<unknown>;
  readonly now?: () => Date;
}): { gateway: ReceiptGateway; invokeCommand: ReturnType<typeof vi.fn<InvokeCommand>> } {
  const invokeCommand = vi.fn<InvokeCommand>().mockImplementation(options.response);
  const gateway = createTauriReceiptGateway({
    invokeCommand,
    now: options.now ?? ((): Date => LOCAL_NOON),
  });
  return { gateway, invokeCommand };
}

// Un rejet de commande Tauri porte `{ code, message }` ; un payload refusé par Tauri, une chaîne.
const COMMAND_REJECTIONS: readonly { description: string; reason: unknown }[] = [
  { description: "a command error", reason: { code: "template", message: "unknown template" } },
  { description: "a payload rejected by tauri", reason: "invalid args `input` for command" },
];

describe("createTauriReceiptGateway", () => {
  afterEach(() => {
    vi.unstubAllEnvs();
  });

  describe("listTemplates", () => {
    it("invokes list_templates without arguments", async () => {
      // Arrange
      const { gateway, invokeCommand } = createGateway({
        response: () => Promise.resolve(TEMPLATES),
      });

      // Act
      await gateway.listTemplates();

      // Assert
      expect(invokeCommand).toHaveBeenCalledExactlyOnceWith("list_templates", {});
    });

    it("returns the templates in the backend order", async () => {
      // Arrange
      const { gateway } = createGateway({ response: () => Promise.resolve(TEMPLATES) });

      // Act
      const templates = await gateway.listTemplates();

      // Assert
      expect(templates).toEqual(TEMPLATES);
    });

    it("returns an empty list when the backend lists no template", async () => {
      // Arrange
      const { gateway } = createGateway({ response: () => Promise.resolve([]) });

      // Act
      const templates = await gateway.listTemplates();

      // Assert
      expect(templates).toEqual([]);
    });

    it.each<{ description: string; response: unknown }>([
      { description: "null", response: null },
      { description: "a single template", response: { id: "standard", label: "Standard" } },
      { description: "a null entry", response: [null] },
      { description: "an entry without label", response: [{ id: "standard" }] },
      { description: "an entry without id", response: [{ label: "Standard" }] },
      { description: "a non string id", response: [{ id: 1, label: "Standard" }] },
      { description: "a non string label", response: [{ id: "standard", label: null }] },
      {
        description: "an inherited id",
        response: [Object.assign(Object.create({ id: "standard" }), { label: "Standard" })],
      },
    ])("rejects $description as an unknown error", async ({ response }) => {
      // Arrange
      const { gateway } = createGateway({ response: () => Promise.resolve(response) });

      // Act
      const result = gateway.listTemplates();

      // Assert
      await expect(result).rejects.toThrow(INVALID_RESPONSE_ERROR_CODE);
      expect(toCommandErrorCode(await result.catch((reason: unknown) => reason))).toBe("unknown");
    });

    it.each(COMMAND_REJECTIONS)("passes $description through unchanged", async ({ reason }) => {
      // Arrange
      const { gateway } = createGateway({ response: () => Promise.reject(reason) });

      // Act
      const result = gateway.listTemplates();

      // Assert
      await expect(result).rejects.toBe(reason);
    });
  });

  describe("renderPreview", () => {
    it("invokes preview_receipt with the template, the input and the local issue date", async () => {
      // Arrange
      const request = previewRequest();
      const { gateway, invokeCommand } = createGateway({
        response: () => Promise.resolve(PREVIEW_HTML),
      });

      // Act
      await gateway.renderPreview(request);

      // Assert
      expect(invokeCommand).toHaveBeenCalledExactlyOnceWith("preview_receipt", {
        templateId: "standard",
        input: sampleReceiptInput(),
        issueDate: "2026-10-05",
      });
    });

    it("returns the receipt html", async () => {
      // Arrange
      const { gateway } = createGateway({ response: () => Promise.resolve(PREVIEW_HTML) });

      // Act
      const html = await gateway.renderPreview(previewRequest());

      // Assert
      expect(html).toBe(PREVIEW_HTML);
    });

    it("dates the receipt with the local day just after midnight", async () => {
      // Arrange
      vi.stubEnv("TZ", "Europe/Paris");
      const { gateway, invokeCommand } = createGateway({
        response: () => Promise.resolve(PREVIEW_HTML),
        now: () => new Date("2026-10-05T22:05:00Z"),
      });

      // Act
      await gateway.renderPreview(previewRequest());

      // Assert
      expect(invokeCommand).toHaveBeenCalledWith(
        "preview_receipt",
        expect.objectContaining({ issueDate: "2026-10-06" }),
      );
    });

    it.each<{ description: string; response: unknown }>([
      { description: "null", response: null },
      { description: "a number", response: 42 },
      { description: "an object", response: { html: PREVIEW_HTML } },
    ])("rejects $description as an unknown error", async ({ response }) => {
      // Arrange
      const { gateway } = createGateway({ response: () => Promise.resolve(response) });

      // Act
      const result = gateway.renderPreview(previewRequest());

      // Assert
      await expect(result).rejects.toThrow(INVALID_RESPONSE_ERROR_CODE);
      expect(toCommandErrorCode(await result.catch((reason: unknown) => reason))).toBe("unknown");
    });

    it.each(COMMAND_REJECTIONS)("passes $description through unchanged", async ({ reason }) => {
      // Arrange
      const { gateway } = createGateway({ response: () => Promise.reject(reason) });

      // Act
      const result = gateway.renderPreview(previewRequest());

      // Assert
      await expect(result).rejects.toBe(reason);
    });
  });

  describe("sendReceipt", () => {
    it("invokes send_receipt with the template, the input and the local issue date", async () => {
      // Arrange
      const request = previewRequest();
      const { gateway, invokeCommand } = createGateway({ response: () => Promise.resolve(null) });

      // Act
      await gateway.sendReceipt(request);

      // Assert
      expect(invokeCommand).toHaveBeenCalledExactlyOnceWith("send_receipt", {
        templateId: "standard",
        input: sampleReceiptInput(),
        issueDate: "2026-10-05",
      });
    });

    it("resolves once the backend confirms the sending", async () => {
      // Arrange
      const { gateway } = createGateway({ response: () => Promise.resolve(null) });

      // Act
      const result = gateway.sendReceipt(previewRequest());

      // Assert
      await expect(result).resolves.toBeUndefined();
    });

    it.each<{ description: string; response: unknown }>([
      { description: "undefined", response: undefined },
      { description: "a string", response: "sent" },
      { description: "an object", response: {} },
    ])("rejects $description as an unknown error", async ({ response }) => {
      // Arrange
      const { gateway } = createGateway({ response: () => Promise.resolve(response) });

      // Act
      const result = gateway.sendReceipt(previewRequest());

      // Assert
      await expect(result).rejects.toThrow(INVALID_RESPONSE_ERROR_CODE);
      expect(toCommandErrorCode(await result.catch((reason: unknown) => reason))).toBe("unknown");
    });

    it.each(COMMAND_REJECTIONS)("passes $description through unchanged", async ({ reason }) => {
      // Arrange
      const { gateway } = createGateway({ response: () => Promise.reject(reason) });

      // Act
      const result = gateway.sendReceipt(previewRequest());

      // Assert
      await expect(result).rejects.toBe(reason);
    });
  });

  it("reads the clock at each call to date the receipt", async () => {
    // Arrange
    let currentTime = new Date(2026, 9, 5, 23, 59);
    const { gateway, invokeCommand } = createGateway({
      response: () => Promise.resolve(PREVIEW_HTML),
      now: () => currentTime,
    });
    await gateway.renderPreview(previewRequest());
    currentTime = new Date(2026, 9, 6, 0, 1);

    // Act
    await gateway.renderPreview(previewRequest());

    // Assert
    expect(invokeCommand.mock.calls.map(([, args]) => args["issueDate"])).toEqual([
      "2026-10-05",
      "2026-10-06",
    ]);
  });
});
