/**
 * Date calendaire locale de `date` au format ISO `YYYY-MM-DD`.
 * Lève une erreur pour une date invalide.
 */
export function toLocalIsoDate(date: Date): string {
  if (Number.isNaN(date.getTime())) {
    throw new Error("[UI_INVALID_CLOCK_DATE] the clock returned an invalid date");
  }
  // `toISOString` donnerait la date UTC, décalée d'un jour autour de minuit local.
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${date.getFullYear()}-${month}-${day}`;
}
