import { describe, expect, it, vi } from "vitest";

import type { TenantProfileEntry, TenantProfilePayload } from "./api";
import type { InvokeCommand } from "./invoke-command";
import { createTauriTenantProfileGateway } from "./tauri-tenant-profile-gateway";
import type { TenantProfileGateway } from "./tenant-profile-gateway";
import { toCommandErrorCode } from "./to-command-error-code";

const TENANT_PROFILES: readonly TenantProfileEntry[] = [
  { id: "jeanne-martin", label: "Jeanne Martin" },
  { id: "leo-durand", label: "Léo Durand" },
];

const JEANNE_ENTRY: TenantProfileEntry = { id: "jeanne-martin", label: "Jeanne Martin" };

// Code de log porté par l'erreur d'une réponse du backend de forme inattendue.
const INVALID_RESPONSE_ERROR_CODE = "[UI_INVALID_COMMAND_RESPONSE]";

function sampleProfile(): TenantProfilePayload {
  return {
    templateId: "standard",
    tenantName: "Jeanne Martin",
    tenantAddress: "8 rue Victor Hugo, 75011 Paris",
    tenantEmail: "jeanne.martin@example.fr",
    propertyAddress: "12 rue des Lilas, 75011 Paris",
    rentCents: 65_000,
    chargesCents: 5_050,
  };
}

function createGateway(response: () => Promise<unknown>): {
  gateway: TenantProfileGateway;
  invokeCommand: ReturnType<typeof vi.fn<InvokeCommand>>;
} {
  const invokeCommand = vi.fn<InvokeCommand>().mockImplementation(response);
  return { gateway: createTauriTenantProfileGateway({ invokeCommand }), invokeCommand };
}

async function rejectionOf(result: Promise<unknown>): Promise<unknown> {
  return result.then(
    () => undefined,
    (reason: unknown) => reason,
  );
}

// Un rejet de commande Tauri porte `{ code, message }` ; un payload refusé par Tauri, une chaîne.
const COMMAND_REJECTIONS: readonly { description: string; reason: unknown }[] = [
  {
    description: "a command error",
    reason: { code: "tenantProfileExists", message: "jeanne-martin.toml existe déjà" },
  },
  { description: "a payload rejected by tauri", reason: "invalid args `profile` for command" },
];

// Réponses de forme inattendue pour une commande qui renvoie une `TenantProfileEntry`.
const MALFORMED_ENTRIES: readonly { description: string; response: unknown }[] = [
  { description: "null", response: null },
  { description: "a list of entries", response: [JEANNE_ENTRY] },
  { description: "an entry without label", response: { id: "jeanne-martin" } },
  { description: "an entry without id", response: { label: "Jeanne Martin" } },
  { description: "a non string id", response: { id: 1, label: "Jeanne Martin" } },
  {
    description: "an inherited id",
    response: Object.assign(Object.create({ id: "jeanne-martin" }), { label: "Jeanne Martin" }),
  },
];

// Commandes d'enregistrement : même payload, même réponse, seul le nom de commande change.
const SAVE_COMMANDS: readonly {
  command: string;
  save: (gateway: TenantProfileGateway, profile: TenantProfilePayload) => Promise<unknown>;
}[] = [
  {
    command: "create_tenant_profile",
    save: (gateway, profile) => gateway.createTenantProfile(profile),
  },
  {
    command: "replace_tenant_profile",
    save: (gateway, profile) => gateway.replaceTenantProfile(profile),
  },
];

describe("createTauriTenantProfileGateway", () => {
  describe("listTenantProfiles", () => {
    it("invokes list_tenant_profiles without arguments", async () => {
      // Arrange
      const { gateway, invokeCommand } = createGateway(() => Promise.resolve(TENANT_PROFILES));

      // Act
      await gateway.listTenantProfiles();

      // Assert
      expect(invokeCommand).toHaveBeenCalledExactlyOnceWith("list_tenant_profiles", {});
    });

    it("returns the tenant profiles in the backend order", async () => {
      // Arrange
      const { gateway } = createGateway(() => Promise.resolve(TENANT_PROFILES));

      // Act
      const entries = await gateway.listTenantProfiles();

      // Assert
      expect(entries).toEqual(TENANT_PROFILES);
    });

    it("returns an empty list when no tenant profile is saved", async () => {
      // Arrange
      const { gateway } = createGateway(() => Promise.resolve([]));

      // Act
      const entries = await gateway.listTenantProfiles();

      // Assert
      expect(entries).toEqual([]);
    });

    it.each<{ description: string; response: unknown }>([
      { description: "null", response: null },
      { description: "a single entry", response: JEANNE_ENTRY },
      { description: "a null entry", response: [null] },
      { description: "an entry without label", response: [{ id: "jeanne-martin" }] },
      { description: "a non string label", response: [{ id: "jeanne-martin", label: 3 }] },
    ])("rejects $description as an unknown error", async ({ response }) => {
      // Arrange
      const { gateway } = createGateway(() => Promise.resolve(response));

      // Act
      const reason = await rejectionOf(gateway.listTenantProfiles());

      // Assert
      expect(reason).toBeInstanceOf(Error);
      expect((reason as Error).message).toContain(INVALID_RESPONSE_ERROR_CODE);
      expect(toCommandErrorCode(reason)).toBe("unknown");
    });

    it.each(COMMAND_REJECTIONS)("passes $description through unchanged", async ({ reason }) => {
      // Arrange
      const { gateway } = createGateway(() => Promise.reject(reason));

      // Act
      const result = gateway.listTenantProfiles();

      // Assert
      await expect(result).rejects.toBe(reason);
    });
  });

  describe("loadTenantProfile", () => {
    it("invokes load_tenant_profile with the profile id", async () => {
      // Arrange
      const { gateway, invokeCommand } = createGateway(() => Promise.resolve(sampleProfile()));

      // Act
      await gateway.loadTenantProfile("jeanne-martin");

      // Assert
      expect(invokeCommand).toHaveBeenCalledExactlyOnceWith("load_tenant_profile", {
        id: "jeanne-martin",
      });
    });

    it("returns the tenant profile", async () => {
      // Arrange
      const { gateway } = createGateway(() => Promise.resolve(sampleProfile()));

      // Act
      const profile = await gateway.loadTenantProfile("jeanne-martin");

      // Assert
      expect(profile).toEqual(sampleProfile());
    });

    it("keeps only the fields of a tenant profile", async () => {
      // Arrange
      const { gateway } = createGateway(() =>
        Promise.resolve({ ...sampleProfile(), landlordName: "Paul Bernard" }),
      );

      // Act
      const profile = await gateway.loadTenantProfile("jeanne-martin");

      // Assert
      expect(profile).toEqual(sampleProfile());
    });

    it.each<{ description: string; response: unknown }>([
      { description: "null", response: null },
      { description: "a string", response: "jeanne-martin" },
      {
        description: "a profile without template",
        response: { ...sampleProfile(), templateId: undefined },
      },
      { description: "a non string tenant name", response: { ...sampleProfile(), tenantName: 1 } },
      {
        description: "a profile without email",
        response: { ...sampleProfile(), tenantEmail: undefined },
      },
      {
        description: "a rent given as text",
        response: { ...sampleProfile(), rentCents: "650,00" },
      },
      { description: "a fractional rent", response: { ...sampleProfile(), rentCents: 650.5 } },
      { description: "negative charges", response: { ...sampleProfile(), chargesCents: -1 } },
      {
        description: "charges beyond the safe integers",
        response: { ...sampleProfile(), chargesCents: Number.MAX_SAFE_INTEGER + 1 },
      },
      {
        description: "an inherited address",
        response: Object.assign(Object.create({ propertyAddress: "12 rue des Lilas" }), {
          ...sampleProfile(),
          propertyAddress: undefined,
        }),
      },
    ])("rejects $description as an unknown error", async ({ response }) => {
      // Arrange
      const { gateway } = createGateway(() => Promise.resolve(response));

      // Act
      const reason = await rejectionOf(gateway.loadTenantProfile("jeanne-martin"));

      // Assert
      expect(reason).toBeInstanceOf(Error);
      expect((reason as Error).message).toContain(INVALID_RESPONSE_ERROR_CODE);
      expect(toCommandErrorCode(reason)).toBe("unknown");
    });

    it.each(COMMAND_REJECTIONS)("passes $description through unchanged", async ({ reason }) => {
      // Arrange
      const { gateway } = createGateway(() => Promise.reject(reason));

      // Act
      const result = gateway.loadTenantProfile("jeanne-martin");

      // Assert
      await expect(result).rejects.toBe(reason);
    });
  });

  describe.each(SAVE_COMMANDS)("$command", ({ command, save }) => {
    it("invokes the command with the profile", async () => {
      // Arrange
      const { gateway, invokeCommand } = createGateway(() => Promise.resolve(JEANNE_ENTRY));

      // Act
      await save(gateway, sampleProfile());

      // Assert
      expect(invokeCommand).toHaveBeenCalledExactlyOnceWith(command, { profile: sampleProfile() });
    });

    it("returns the entry of the saved profile", async () => {
      // Arrange
      const { gateway } = createGateway(() => Promise.resolve(JEANNE_ENTRY));

      // Act
      const entry = await save(gateway, sampleProfile());

      // Assert
      expect(entry).toEqual(JEANNE_ENTRY);
    });

    it.each(MALFORMED_ENTRIES)("rejects $description as an unknown error", async ({ response }) => {
      // Arrange
      const { gateway } = createGateway(() => Promise.resolve(response));

      // Act
      const reason = await rejectionOf(save(gateway, sampleProfile()));

      // Assert
      expect(reason).toBeInstanceOf(Error);
      expect((reason as Error).message).toContain(INVALID_RESPONSE_ERROR_CODE);
      expect(toCommandErrorCode(reason)).toBe("unknown");
    });

    it.each(COMMAND_REJECTIONS)("passes $description through unchanged", async ({ reason }) => {
      // Arrange
      const { gateway } = createGateway(() => Promise.reject(reason));

      // Act
      const result = save(gateway, sampleProfile());

      // Assert
      await expect(result).rejects.toBe(reason);
    });
  });

  it.each<{
    description: string;
    call: (gateway: TenantProfileGateway) => Promise<unknown>;
    response: unknown;
    content: string;
  }>([
    {
      description: "profile labels",
      call: (gateway) => gateway.listTenantProfiles(),
      response: [JEANNE_ENTRY, { id: 1 }],
      content: "Jeanne Martin",
    },
    {
      description: "profile content",
      call: (gateway) => gateway.loadTenantProfile("jeanne-martin"),
      response: { ...sampleProfile(), rentCents: "650,00" },
      content: "jeanne.martin@example.fr",
    },
    {
      description: "saved entry",
      call: (gateway) => gateway.createTenantProfile(sampleProfile()),
      response: "saved Jeanne Martin",
      content: "Jeanne Martin",
    },
  ])(
    "keeps the $description of a malformed response out of the error",
    async ({ call, response, content }) => {
      // Arrange
      const { gateway } = createGateway(() => Promise.resolve(response));

      // Act
      const reason = await rejectionOf(call(gateway));

      // Assert
      expect(reason).toBeInstanceOf(Error);
      expect((reason as Error).message).not.toContain(content);
    },
  );
});
