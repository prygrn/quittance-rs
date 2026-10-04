import type { TemplateInfo } from "./api";
import type { PreviewRequest } from "./screen-state";

/**
 * Accès aux commandes du backend, injecté dans l'écran pour le tester avec des fakes.
 * Une commande en échec rejette sa promesse avec une valeur portant `code` (un
 * `CommandErrorCode`) ; toute autre valeur de rejet est traitée comme `unknown`.
 */
export interface ReceiptGateway {
  /** Modèles de quittance disponibles, dans l'ordre d'affichage. */
  listTemplates(): Promise<readonly TemplateInfo[]>;
  /** HTML complet de la quittance, affiché dans une `iframe srcdoc`. */
  renderPreview(request: PreviewRequest): Promise<string>;
  /** Génère le PDF et l'envoie au locataire, avec copie cachée au bailleur. */
  sendReceipt(request: PreviewRequest): Promise<void>;
}
