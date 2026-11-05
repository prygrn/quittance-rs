import { parseAmountCents } from "./amount";
import type { TenantProfilePayload } from "./api";
import { isWellFormedEmail } from "./email";
import type { FieldErrorCode, ReceiptFormValues } from "./receipt-form";

/** Champs du formulaire repris par une fiche locataire : ni la période ni la date de paiement. */
export type TenantProfileFormField =
  | "tenantName"
  | "tenantAddress"
  | "tenantEmail"
  | "propertyAddress"
  | "rentAmount"
  | "chargesAmount";

export type TenantProfileFormValues = Pick<ReceiptFormValues, TenantProfileFormField>;

/** Contenu d'une fiche hors modèle de quittance, choisi à part dans sa liste. */
export type TenantProfileFields = Omit<TenantProfilePayload, "templateId">;

export type TenantProfileFormErrors = Partial<Record<TenantProfileFormField, FieldErrorCode>>;

export type TenantProfileFormResult =
  | { isValid: true; fields: TenantProfileFields }
  | { isValid: false; errors: TenantProfileFormErrors };

type AmountField = "rentAmount" | "chargesAmount";

const TENANT_PROFILE_FORM_FIELDS: readonly TenantProfileFormField[] = [
  "tenantName",
  "tenantAddress",
  "tenantEmail",
  "propertyAddress",
  "rentAmount",
  "chargesAmount",
];
const AMOUNT_FIELDS: readonly AmountField[] = ["rentAmount", "chargesAmount"];

/**
 * Nettoie et valide les champs d'une fiche ; renvoie soit son contenu, soit une erreur par
 * champ. `validateReceiptForm` s'appuie dessus pour ces mêmes champs.
 */
export function validateTenantProfileForm(
  values: TenantProfileFormValues,
): TenantProfileFormResult {
  // Liste blanche : seuls les champs connus du formulaire sont recopiés.
  const trimmed: TenantProfileFormValues = {
    tenantName: values.tenantName.trim(),
    tenantAddress: values.tenantAddress.trim(),
    tenantEmail: values.tenantEmail.trim(),
    propertyAddress: values.propertyAddress.trim(),
    rentAmount: values.rentAmount.trim(),
    chargesAmount: values.chargesAmount.trim(),
  };
  const errors: TenantProfileFormErrors = {};

  // Un champ vide n'a qu'une erreur à afficher : « obligatoire » prime sur le format.
  for (const field of TENANT_PROFILE_FORM_FIELDS) {
    if (trimmed[field] === "") {
      errors[field] = "missing";
    }
  }
  if (errors.tenantEmail === undefined && !isWellFormedEmail(trimmed.tenantEmail)) {
    errors.tenantEmail = "invalidEmail";
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
    amountCents.rentAmount === undefined ||
    amountCents.chargesAmount === undefined
  ) {
    return { isValid: false, errors };
  }
  return {
    isValid: true,
    fields: {
      tenantName: trimmed.tenantName,
      tenantAddress: trimmed.tenantAddress,
      tenantEmail: trimmed.tenantEmail,
      propertyAddress: trimmed.propertyAddress,
      rentCents: amountCents.rentAmount,
      chargesCents: amountCents.chargesAmount,
    },
  };
}
