import "./receipt-screen.css";
import { mountReceiptScreen } from "./mount-receipt-screen";
import { UNCONNECTED_GATEWAY } from "./unconnected-gateway";

mountReceiptScreen({ root: document, gateway: UNCONNECTED_GATEWAY }).catch((error: unknown) => {
  console.error("[UI_MOUNT_FAILED] receipt screen could not be mounted", error);
});
