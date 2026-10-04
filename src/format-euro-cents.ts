const EURO_FORMAT = new Intl.NumberFormat("fr-FR", { style: "currency", currency: "EUR" });
const CENTS_PER_EURO = 100;

/** Formate un montant en centimes à la française, par exemple « 1 234,56 € ». */
export function formatEuroCents(cents: number): string {
  return EURO_FORMAT.format(cents / CENTS_PER_EURO);
}
