const ISO_DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/;

/** Vrai pour une date calendaire existante au format `YYYY-MM-DD`. */
export function isValidIsoDate(value: string): boolean {
  if (!ISO_DATE_PATTERN.test(value)) {
    return false;
  }
  // `Date` reporte les jours en trop sur le mois suivant (30 février → 2 mars) :
  // l'aller-retour vers l'ISO écarte les dates inexistantes.
  const timestamp = Date.parse(value);
  return !Number.isNaN(timestamp) && new Date(timestamp).toISOString().startsWith(value);
}
