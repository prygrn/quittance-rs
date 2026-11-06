import { fileURLToPath } from "node:url";

/**
 * Build release produit par `tauri build --no-bundle` (cible `make test-e2e`), front
 * embarqué : l'exécutable lancé par tauri-driver et le dossier `fiches/` qu'il lit à côté
 * de lui.
 */
export const RELEASE_BUILD = {
  applicationPath: fileURLToPath(new URL("../../target/release/quittance-app", import.meta.url)),
  tenantProfileDirectory: fileURLToPath(new URL("../../target/release/fiches", import.meta.url)),
} as const;
