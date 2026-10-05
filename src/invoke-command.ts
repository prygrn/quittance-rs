/**
 * Appel d'une commande du backend, au format de `invoke` de `@tauri-apps/api/core` :
 * arguments en camelCase, promesse rejetée avec la valeur d'erreur de la commande.
 * Injecté dans la passerelle pour la tester avec un fake.
 */
export type InvokeCommand = (
  command: string,
  args: Readonly<Record<string, unknown>>,
) => Promise<unknown>;
