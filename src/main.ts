import { invoke } from "@tauri-apps/api/core";

import "./receipt-screen.css";
import { mountReceiptScreen } from "./mount-receipt-screen";
import type { ReceiptGateway } from "./receipt-gateway";
import { createTauriReceiptGateway } from "./tauri-receipt-gateway";
import { createTauriTenantProfileGateway } from "./tauri-tenant-profile-gateway";
import type { TenantProfileGateway } from "./tenant-profile-gateway";

const gateway: ReceiptGateway = createTauriReceiptGateway({
  invokeCommand: invoke,
  now: (): Date => new Date(),
});

const tenantProfileGateway: TenantProfileGateway = createTauriTenantProfileGateway({
  invokeCommand: invoke,
});

mountReceiptScreen({ root: document, gateway, tenantProfileGateway }).catch(
  (error: unknown): void => {
    console.error("[UI_MOUNT_FAILED] receipt screen could not be mounted", error);
  },
);
