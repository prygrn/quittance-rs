const EURO_FORMAT = {
  FORMATTER: new Intl.NumberFormat("fr-FR", { style: "currency", currency: "EUR" }),
  CENTS_PER_EURO: 100,
} as const;

/** Formate un montant en centimes à la française, par exemple « 1 234,56 € ». */
export function formatEuroCents(cents: number): string {
  return EURO_FORMAT.FORMATTER.format(cents / EURO_FORMAT.CENTS_PER_EURO);
}
