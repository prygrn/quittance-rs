import { type AmountErrorCode, parseAmountCents } from "./amount";
import type { ReceiptInput } from "./api";
import { isWellFormedEmail } from "./email";
import { isValidIsoDate } from "./iso-date";

/** Valeurs brutes du formulaire, telles que lues dans les champs HTML. */
export type ReceiptFormValues = Record<keyof ReceiptInput, string>;

export type ReceiptFormField = keyof ReceiptFormValues;

export type FieldErrorCode =
  "missing" | "invalidEmail" | "invalidDate" | "invertedPeriod" | AmountErrorCode;

export type ReceiptFormErrors = Partial<Record<ReceiptFormField, FieldErrorCode>>;

export type ReceiptFormResult =
  { isValid: true; input: ReceiptInput } | { isValid: false; errors: ReceiptFormErrors };

type DateField = "periodStart" | "periodEnd" | "paymentDate";
type AmountField = "rentCents" | "chargesCents";

const DATE_FIELDS: readonly DateField[] = ["periodStart", "periodEnd", "paymentDate"];
const AMOUNT_FIELDS: readonly AmountField[] = ["rentCents", "chargesCents"];

/** Nettoie et valide la saisie ; renvoie soit un `ReceiptInput`, soit une erreur par champ. */
export function validateReceiptForm(values: ReceiptFormValues): ReceiptFormResult {
  const trimmed = trimmedValues(values);
  const errors: ReceiptFormErrors = {};

  // Un champ vide n'a qu'une erreur à afficher : « obligatoire » prime sur le format.
  for (const field of Object.keys(trimmed) as ReceiptFormField[]) {
    if (trimmed[field] === "") {
      errors[field] = "missing";
    }
  }
  if (errors.tenantEmail === undefined && !isWellFormedEmail(trimmed.tenantEmail)) {
    errors.tenantEmail = "invalidEmail";
  }
  for (const field of DATE_FIELDS) {
    if (errors[field] === undefined && !isValidIsoDate(trimmed[field])) {
      errors[field] = "invalidDate";
    }
  }
  // Les dates ISO valides se comparent dans l'ordre lexicographique.
  const hasValidPeriodBounds = errors.periodStart === undefined && errors.periodEnd === undefined;
  if (hasValidPeriodBounds && trimmed.periodStart > trimmed.periodEnd) {
    errors.periodEnd = "invertedPeriod";
  }
  const amountCents: Partial<Record<AmountField, number>> = {};
  for (const field of AMOUNT_FIELDS) {
    if (errors[field] !== undefined) {
      continue;
    }
    const amount = parseAmountCents(trimmed[field]);
    if (amount.isValid) {
      amountCents[field] = amount.cents;
    } else {
      errors[field] = amount.error;
    }
  }

  if (
    Object.keys(errors).length > 0 ||
    amountCents.rentCents === undefined ||
    amountCents.chargesCents === undefined
  ) {
    return { isValid: false, errors };
  }
  return {
    isValid: true,
    input: { ...trimmed, rentCents: amountCents.rentCents, chargesCents: amountCents.chargesCents },
  };
}

function trimmedValues(values: ReceiptFormValues): ReceiptFormValues {
  // Liste blanche : seuls les champs connus de `ReceiptInput` sont recopiés.
  return {
    tenantName: values.tenantName.trim(),
    tenantAddress: values.tenantAddress.trim(),
    tenantEmail: values.tenantEmail.trim(),
    propertyAddress: values.propertyAddress.trim(),
    periodStart: values.periodStart.trim(),
    periodEnd: values.periodEnd.trim(),
    rentCents: values.rentCents.trim(),
    chargesCents: values.chargesCents.trim(),
    paymentDate: values.paymentDate.trim(),
  };
}
