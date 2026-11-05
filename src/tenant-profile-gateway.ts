import type { TenantProfileEntry, TenantProfilePayload } from "./api";

/**
 * Accès aux commandes du backend sur les fiches locataire, injecté dans l'écran pour le
 * tester avec des fakes. Une commande en échec rejette sa promesse comme celles de
 * `ReceiptGateway`.
 */
export interface TenantProfileGateway {
  /** Fiches enregistrées, triées par libellé. */
  listTenantProfiles(): Promise<readonly TenantProfileEntry[]>;
  loadTenantProfile(id: string): Promise<TenantProfilePayload>;
  /** Rejette avec le code `tenantProfileExists` quand une fiche porte déjà cet id. */
  createTenantProfile(profile: TenantProfilePayload): Promise<TenantProfileEntry>;
  /** Écrase la fiche de même id, après confirmation de l'utilisateur. */
  replaceTenantProfile(profile: TenantProfilePayload): Promise<TenantProfileEntry>;
}
