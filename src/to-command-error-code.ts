import type { CommandErrorCode } from "./api";

/** Extrait le code d'erreur du rejet d'une commande ; tout rejet non reconnu devient `unknown`. */
export function toCommandErrorCode(reason: unknown): CommandErrorCode {
  void reason;
  throw new Error("not implemented");
}
