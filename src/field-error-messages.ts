import type { FieldErrorCode } from "./receipt-form";

/** Message affiché sous un champ pour chaque erreur de saisie, repris de la maquette. */
export const FIELD_ERROR_MESSAGES: Readonly<Record<FieldErrorCode, string>> = {
  missing: "",
  invalidEmail: "",
  invalidDate: "",
  invertedPeriod: "",
  invalidAmount: "",
  negativeAmount: "",
  tooManyDecimals: "",
  amountTooLarge: "",
};
