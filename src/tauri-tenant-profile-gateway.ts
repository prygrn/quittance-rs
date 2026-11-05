import type { TenantProfileEntry, TenantProfilePayload } from "./api";
import { invalidCommandResponseError } from "./invalid-command-response-error";
import type { InvokeCommand } from "./invoke-command";
import { readOwnField } from "./read-own-field";
import type { TenantProfileGateway } from "./tenant-profile-gateway";

/**
 * Passerelle vers les commandes Tauri `list_tenant_profiles`, `load_tenant_profile`,
 * `create_tenant_profile` et `replace_tenant_profile`.
 */
export function createTauriTenantProfileGateway(dependencies: {
  readonly invokeCommand: InvokeCommand;
}): TenantProfileGateway {
  const { invokeCommand } = dependencies;
  const saveTenantProfile = async (command: {
    readonly name: string;
    readonly profile: TenantProfilePayload;
  }): Promise<TenantProfileEntry> => {
    const response = await invokeCommand(command.name, { profile: command.profile });
    const entry = toTenantProfileEntry(response);
    if (entry === null) {
      throw invalidCommandResponseError({
        command: command.name,
        expected: "TenantProfileEntry",
        response,
      });
    }
    return entry;
  };

  return {
    listTenantProfiles: async (): Promise<readonly TenantProfileEntry[]> => {
      const response = await invokeCommand("list_tenant_profiles", {});
      const entries = Array.isArray(response) ? response.map(toTenantProfileEntry) : null;
      if (entries === null || entries.includes(null)) {
        throw invalidCommandResponseError({
          command: "list_tenant_profiles",
          expected: "TenantProfileEntry[]",
          response,
        });
      }
      return entries.filter((entry): entry is TenantProfileEntry => entry !== null);
    },
    loadTenantProfile: async (id: string): Promise<TenantProfilePayload> => {
      const response = await invokeCommand("load_tenant_profile", { id });
      const profile = toTenantProfilePayload(response);
      if (profile === null) {
        throw invalidCommandResponseError({
          command: "load_tenant_profile",
          expected: "TenantProfilePayload",
          response,
        });
      }
      return profile;
    },
    createTenantProfile: (profile: TenantProfilePayload): Promise<TenantProfileEntry> =>
      saveTenantProfile({ name: "create_tenant_profile", profile }),
    replaceTenantProfile: (profile: TenantProfilePayload): Promise<TenantProfileEntry> =>
      saveTenantProfile({ name: "replace_tenant_profile", profile }),
  };
}

function toTenantProfileEntry(value: unknown): TenantProfileEntry | null {
  const id = readOwnString({ value, field: "id" });
  const label = readOwnString({ value, field: "label" });
  return id === null || label === null ? null : { id, label };
}

// Recopie champ par champ : un champ en trop dans la réponse ne remonte pas jusqu'à l'écran.
function toTenantProfilePayload(value: unknown): TenantProfilePayload | null {
  const templateId = readOwnString({ value, field: "templateId" });
  const tenantName = readOwnString({ value, field: "tenantName" });
  const tenantAddress = readOwnString({ value, field: "tenantAddress" });
  const tenantEmail = readOwnString({ value, field: "tenantEmail" });
  const propertyAddress = readOwnString({ value, field: "propertyAddress" });
  const rentCents = readOwnCents({ value, field: "rentCents" });
  const chargesCents = readOwnCents({ value, field: "chargesCents" });
  if (
    templateId === null ||
    tenantName === null ||
    tenantAddress === null ||
    tenantEmail === null ||
    propertyAddress === null ||
    rentCents === null ||
    chargesCents === null
  ) {
    return null;
  }
  return {
    templateId,
    tenantName,
    tenantAddress,
    tenantEmail,
    propertyAddress,
    rentCents,
    chargesCents,
  };
}

function readOwnString(candidate: {
  readonly value: unknown;
  readonly field: string;
}): string | null {
  const field = readOwnField(candidate);
  return typeof field === "string" ? field : null;
}

function readOwnCents(candidate: {
  readonly value: unknown;
  readonly field: string;
}): number | null {
  const field = readOwnField(candidate);
  return typeof field === "number" && Number.isSafeInteger(field) && field >= 0 ? field : null;
}
