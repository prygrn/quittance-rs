// Types miroirs des commandes Tauri. Les noms de champs supposent que F6 sérialise
// les structures Rust avec serde `rename_all = "camelCase"`.

/** Saisie d'une quittance, déjà nettoyée et typée côté client. */
export interface ReceiptInput {
  tenantName: string;
  tenantAddress: string;
  tenantEmail: string;
  propertyAddress: string;
  /** Date ISO `YYYY-MM-DD`, format natif de `<input type="date">`. */
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
