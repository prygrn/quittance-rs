const FRENCH_DATE_PATTERN = /^(\d{1,2})\/(\d{1,2})\/(\d{4})$/;
const DAY_MONTH_DIGIT_COUNT = 2;

/**
 * Convertit une date saisie `jj/mm/aaaa` en date ISO `YYYY-MM-DD`. Une saisie d'un autre
 * format est renvoyée telle quelle : `validateReceiptForm` la signalera.
 */
export function frenchDateToIsoDate(value: string): string {
  const match = FRENCH_DATE_PATTERN.exec(value.trim());
  if (match === null) {
    return value;
  }
  const [, day = "", month = "", year = ""] = match;
  const pad = (part: string): string => part.padStart(DAY_MONTH_DIGIT_COUNT, "0");
  return `${year}-${pad(month)}-${pad(day)}`;
}
