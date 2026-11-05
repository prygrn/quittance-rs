const AMOUNT_INPUT_FORMAT = {
  DECIMAL_SEPARATOR: ",",
  DECIMAL_DIGIT_COUNT: 2,
  PADDING_DIGIT: "0",
} as const;

/**
 * Écrit un montant en centimes comme une saisie du formulaire, à la française : « 650,00 ».
 * Le découpage des chiffres évite la division flottante, inexacte sur les grands montants.
 */
export function formatAmountInput(cents: number): string {
  if (!Number.isSafeInteger(cents) || cents < 0) {
    throw new Error(
      `[UI_INVALID_AMOUNT_CENTS] expected a non-negative safe integer count of cents, got ${cents}`,
    );
  }
  const digits = String(cents).padStart(
    AMOUNT_INPUT_FORMAT.DECIMAL_DIGIT_COUNT + 1,
    AMOUNT_INPUT_FORMAT.PADDING_DIGIT,
  );
  const integerLength = digits.length - AMOUNT_INPUT_FORMAT.DECIMAL_DIGIT_COUNT;
  return `${digits.slice(0, integerLength)}${AMOUNT_INPUT_FORMAT.DECIMAL_SEPARATOR}${digits.slice(integerLength)}`;
}
