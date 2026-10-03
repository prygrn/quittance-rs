export type AmountErrorCode =
  "invalidAmount" | "negativeAmount" | "tooManyDecimals" | "amountTooLarge";

export type AmountResult =
  { isValid: true; cents: number } | { isValid: false; error: AmountErrorCode };

/** Convertit un montant saisi à la française (« 1 234,56 ») en centimes, sans flottant. */
export function parseAmountCents(rawAmount: string): AmountResult {
  throw new Error("not implemented", { cause: rawAmount });
}
