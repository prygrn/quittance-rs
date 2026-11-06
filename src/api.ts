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
 * Fiche locataire : tout ce qui préremplit une quittance, sauf la période et la date de
 * paiement. Miroir du payload des commandes `load_tenant_profile`, `create_tenant_profile`
 * et `replace_tenant_profile`.
 */
export interface TenantProfilePayload {
  templateId: string;
  tenantName: string;
  tenantAddress: string;
  tenantEmail: string;
  propertyAddress: string;
  /** Montants entiers en centimes, comme dans `ReceiptInput`. */
  rentCents: number;
  chargesCents: number;
}

/** Fiche locataire enregistrée, telle que listée par le backend. */
export interface TenantProfileEntry {
  /** Nom du fichier de la fiche sans l'extension `.toml`, dérivé du nom du locataire. */
  id: string;
  /** Nom du locataire, ou l'`id` quand la fiche est illisible. */
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
 * - `tenantProfile` : fiche locataire absente, illisible ou invalide au chargement, ou
 *   impossible à écrire ;
 * - `tenantProfileExists` : création d'une fiche locataire dont l'id existe déjà ;
 * - `unknown` : toute autre erreur, y compris un code non reconnu par l'UI.
 * Le libellé français de chaque code est choisi par l'UI (F7c).
 */
export type CommandErrorCode =
  | "validation"
  | "template"
  | "signature"
  | "pdf"
  | "mail"
  | "config"
  | "tenantProfile"
  | "tenantProfileExists"
  | "unknown";
