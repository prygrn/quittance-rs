import type { TenantProfilePayload } from "./api";
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

/** Nettoie et valide les champs d'une fiche ; renvoie soit son contenu, soit une erreur par champ. */
export function validateTenantProfileForm(
  values: TenantProfileFormValues,
): TenantProfileFormResult {
  throw new Error(`[UI_NOT_IMPLEMENTED] validateTenantProfileForm(${typeof values})`);
}
