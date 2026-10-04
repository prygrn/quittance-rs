import type { CommandErrorCode } from "./api";

// Enregistrement plutôt que tableau : le compilateur exige chaque code de `CommandErrorCode`.
const KNOWN_COMMAND_ERROR_CODES = {
  validation: true,
  template: true,
  signature: true,
  pdf: true,
  mail: true,
  config: true,
  unknown: true,
} as const satisfies Record<CommandErrorCode, true>;

/** Extrait le code d'erreur du rejet d'une commande ; tout rejet non reconnu devient `unknown`. */
export function toCommandErrorCode(reason: unknown): CommandErrorCode {
  if (typeof reason !== "object" || reason === null || !Object.hasOwn(reason, "code")) {
    return "unknown";
  }
  const code: unknown = (reason as { code: unknown }).code;
  return typeof code === "string" && Object.hasOwn(KNOWN_COMMAND_ERROR_CODES, code)
    ? (code as CommandErrorCode)
    : "unknown";
}
