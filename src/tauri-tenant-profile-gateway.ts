import type { InvokeCommand } from "./invoke-command";
import type { TenantProfileGateway } from "./tenant-profile-gateway";

/**
 * Passerelle vers les commandes Tauri `list_tenant_profiles`, `load_tenant_profile`,
 * `create_tenant_profile` et `replace_tenant_profile`.
 */
export function createTauriTenantProfileGateway(dependencies: {
  readonly invokeCommand: InvokeCommand;
}): TenantProfileGateway {
  throw new Error(`[UI_NOT_IMPLEMENTED] createTauriTenantProfileGateway(${typeof dependencies})`);
}
