import type { AmountErrorCode } from "./amount";
import type { ReceiptInput } from "./api";
import { isValidIsoDate } from "./iso-date";
import { validateTenantProfileForm } from "./tenant-profile-form";

/** Valeurs brutes du formulaire, telles que lues dans les champs HTML. */
export interface ReceiptFormValues {
  tenantName: string;
  tenantAddress: string;
  tenantEmail: string;
  propertyAddress: string;
  periodStart: string;
  periodEnd: string;
  /** Montants saisis en euros, à la française (« 1 234,56 »). */
  rentAmount: string;
  chargesAmount: string;
  paymentDate: string;
}

export type ReceiptFormField = keyof ReceiptFormValues;

export type FieldErrorCode =
  "missing" | "invalidEmail" | "invalidDate" | "invertedPeriod" | AmountErrorCode;

export type ReceiptFormErrors = Partial<Record<ReceiptFormField, FieldErrorCode>>;

export type ReceiptFormResult =
  { isValid: true; input: ReceiptInput } | { isValid: false; errors: ReceiptFormErrors };

type DateField = "periodStart" | "periodEnd" | "paymentDate";

const DATE_FIELDS: readonly DateField[] = ["periodStart", "periodEnd", "paymentDate"];

/**
 * Nettoie et valide la saisie ; renvoie soit un `ReceiptInput`, soit une erreur par champ.
 * Les champs d'une fiche locataire sont validés par `validateTenantProfileForm`.
 */
export function validateReceiptForm(values: ReceiptFormValues): ReceiptFormResult {
  const tenantProfile = validateTenantProfileForm(values);
  const errors: ReceiptFormErrors = tenantProfile.isValid ? {} : { ...tenantProfile.errors };
  const dates: Record<DateField, string> = {
    periodStart: values.periodStart.trim(),
    periodEnd: values.periodEnd.trim(),
    paymentDate: values.paymentDate.trim(),
  };

  for (const field of DATE_FIELDS) {
    if (dates[field] === "") {
      errors[field] = "missing";
    } else if (!isValidIsoDate(dates[field])) {
      errors[field] = "invalidDate";
    }
  }
  // Les dates ISO valides se comparent dans l'ordre lexicographique.
  const hasValidPeriodBounds = errors.periodStart === undefined && errors.periodEnd === undefined;
  if (hasValidPeriodBounds && dates.periodStart > dates.periodEnd) {
    errors.periodEnd = "invertedPeriod";
  }

  if (!tenantProfile.isValid || Object.keys(errors).length > 0) {
    return { isValid: false, errors };
  }
  return { isValid: true, input: { ...tenantProfile.fields, ...dates } };
}
