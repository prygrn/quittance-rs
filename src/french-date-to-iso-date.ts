const FRENCH_DATE_FORMAT = {
  PATTERN: /^(\d{1,2})\/(\d{1,2})\/(\d{4})$/,
  DAY_MONTH_DIGIT_COUNT: 2,
  PADDING_DIGIT: "0",
} as const;

/**
 * Convertit une date saisie `jj/mm/aaaa` en date ISO `YYYY-MM-DD`. Une saisie d'un autre
 * format est renvoyée telle quelle : `validateReceiptForm` la signalera.
 */
export function frenchDateToIsoDate(value: string): string {
  const match = FRENCH_DATE_FORMAT.PATTERN.exec(value.trim());
  if (match === null) {
    return value;
  }
  const [, day = "", month = "", year = ""] = match;
  const pad = (part: string): string =>
    part.padStart(FRENCH_DATE_FORMAT.DAY_MONTH_DIGIT_COUNT, FRENCH_DATE_FORMAT.PADDING_DIGIT);
  return `${year}-${pad(month)}-${pad(day)}`;
}
