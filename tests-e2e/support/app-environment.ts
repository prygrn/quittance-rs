import { fileURLToPath } from "node:url";

import { MAILPIT } from "./mailpit";
import { TEST_LANDLORD } from "./test-landlord";

const SIGNATURE_PATH = fileURLToPath(
  new URL("../../src-tauri/fixtures/signature.png", import.meta.url),
);

function requireChromePath(): string {
  const chromePath = process.env["CHROME_PATH"];
  if (chromePath === undefined || chromePath === "") {
    throw new Error(
      "[E2E_MISSING_CHROME_PATH] CHROME_PATH must point to the Chrome or Chromium binary the app prints PDFs with",
    );
  }
  return chromePath;
}

/**
 * Configuration de l'app pendant les scénarios, passée par l'environnement du processus :
 * elle l'emporte sur un éventuel `.env` posé à côté de l'exécutable, et l'envoi ne peut
 * viser que le Mailpit local, sans chiffrement ni identifiants.
 */
export function appEnvironment(): Readonly<Record<string, string>> {
  return {
    LANDLORD_NAME: TEST_LANDLORD.name,
    LANDLORD_ADDRESS: TEST_LANDLORD.address,
    LANDLORD_EMAIL: TEST_LANDLORD.email,
    LANDLORD_CITY: TEST_LANDLORD.city,
    SIGNATURE_PATH,
    CHROME_PATH: requireChromePath(),
    SMTP_HOST: MAILPIT.host,
    SMTP_PORT: String(MAILPIT.smtpPort),
    SMTP_SECURITY: "none",
  };
}
