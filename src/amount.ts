export type AmountErrorCode =
  "invalidAmount" | "negativeAmount" | "tooManyDecimals" | "amountTooLarge";

export type AmountResult =
  { isValid: true; cents: number } | { isValid: false; error: AmountErrorCode };

const AMOUNT_FORMAT = {
  NEGATIVE_SIGN: "-",
  // Partie entière brute ou groupée par 3 (espace simple, insécable ou fine insécable),
  // puis décimales optionnelles après une virgule ou un point.
  PATTERN: /^(\d+|\d{1,3}(?:[ \u00A0\u202F]\d{3})+)(?:[.,](\d+))?$/,
  GROUP_SEPARATORS: /[ \u00A0\u202F]/g,
  DECIMAL_DIGIT_COUNT: 2,
  PADDING_DIGIT: "0",
} as const;

const MAX_CENTS = BigInt(Number.MAX_SAFE_INTEGER);

/** Convertit un montant saisi à la française (« 1 234,56 ») en centimes, sans flottant. */
export function parseAmountCents(rawAmount: string): AmountResult {
  const amount = rawAmount.trim();
  if (amount.startsWith(AMOUNT_FORMAT.NEGATIVE_SIGN)) {
    return { isValid: false, error: "negativeAmount" };
  }
  const match = AMOUNT_FORMAT.PATTERN.exec(amount);
  if (match === null) {
    return { isValid: false, error: "invalidAmount" };
  }
  const integerDigits = (match[1] ?? "").replace(AMOUNT_FORMAT.GROUP_SEPARATORS, "");
  const decimalDigits = match[2] ?? "";
  if (decimalDigits.length > AMOUNT_FORMAT.DECIMAL_DIGIT_COUNT) {
    return { isValid: false, error: "tooManyDecimals" };
  }
  const paddedDecimals = decimalDigits.padEnd(
    AMOUNT_FORMAT.DECIMAL_DIGIT_COUNT,
    AMOUNT_FORMAT.PADDING_DIGIT,
  );
  // BigInt garde la conversion exacte même au-delà de la précision d'un number.
  const cents = BigInt(integerDigits + paddedDecimals);
  if (cents > MAX_CENTS) {
    return { isValid: false, error: "amountTooLarge" };
  }
  return { isValid: true, cents: Number(cents) };
}
