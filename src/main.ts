import { invoke } from "@tauri-apps/api/core";

import "./receipt-screen.css";
import { mountReceiptScreen } from "./mount-receipt-screen";
import type { ReceiptGateway } from "./receipt-gateway";
import { createTauriReceiptGateway } from "./tauri-receipt-gateway";

const gateway: ReceiptGateway = createTauriReceiptGateway({
  invokeCommand: invoke,
  now: () => new Date(),
});

mountReceiptScreen({ root: document, gateway }).catch((error: unknown) => {
  console.error("[UI_MOUNT_FAILED] receipt screen could not be mounted", error);
});
