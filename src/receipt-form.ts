import type { AmountErrorCode } from "./amount";
import type { ReceiptInput } from "./api";

/** Valeurs brutes du formulaire, telles que lues dans les champs HTML. */
export type ReceiptFormValues = Record<keyof ReceiptInput, string>;

export type ReceiptFormField = keyof ReceiptFormValues;

export type FieldErrorCode =
  "missing" | "invalidEmail" | "invalidDate" | "invertedPeriod" | AmountErrorCode;

export type ReceiptFormErrors = Partial<Record<ReceiptFormField, FieldErrorCode>>;

export type ReceiptFormResult =
  { isValid: true; input: ReceiptInput } | { isValid: false; errors: ReceiptFormErrors };

/** Nettoie et valide la saisie ; renvoie soit un `ReceiptInput`, soit une erreur par champ. */
export function validateReceiptForm(values: ReceiptFormValues): ReceiptFormResult {
  throw new Error("not implemented", { cause: values });
}
