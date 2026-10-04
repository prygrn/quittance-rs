// Types miroirs des commandes Tauri. Les noms de champs supposent que F6 sérialise
// les structures Rust avec serde `rename_all = "camelCase"`.

/** Saisie d'une quittance, déjà nettoyée et typée côté client. */
export interface ReceiptInput {
  tenantName: string;
  tenantAddress: string;
  tenantEmail: string;
  propertyAddress: string;
  /** Date ISO `YYYY-MM-DD`, convertie par l'écran depuis la saisie `jj/mm/aaaa`. */
  periodStart: string;
  periodEnd: string;
  /** Montants entiers en centimes, pour éviter toute erreur d'arrondi. */
  rentCents: number;
  chargesCents: number;
  paymentDate: string;
}

/** Modèle de quittance proposé par le backend. */
export interface TemplateInfo {
  id: string;
  label: string;
}

/**
 * Codes d'erreur renvoyés par les commandes Tauri ; F6 doit sérialiser ses erreurs
 * vers exactement ces valeurs. Liste minimale, une entrée par source d'échec :
 * - `validation` : saisie refusée par `quittance-core` ;
 * - `template` : modèle introuvable ou rendu HTML en échec ;
 * - `signature` : image de signature absente ou illisible ;
 * - `pdf` : génération du PDF en échec ;
 * - `mail` : envoi de l'email en échec ;
 * - `config` : configuration du bailleur ou du serveur mail absente ou invalide ;
 * - `unknown` : toute autre erreur, y compris un code non reconnu par l'UI.
 * Le libellé français de chaque code est choisi par l'UI (F7c).
 */
export type CommandErrorCode =
  "validation" | "template" | "signature" | "pdf" | "mail" | "config" | "unknown";
