import type { ReceiptFormField, ReceiptFormValues } from "./receipt-form";

/** Lit les valeurs brutes des champs, telles que saisies. */
export function readReceiptFormValues(
  fieldInputs: Readonly<Record<ReceiptFormField, HTMLInputElement | HTMLTextAreaElement>>,
): ReceiptFormValues {
  return {
    tenantName: fieldInputs.tenantName.value,
    tenantAddress: fieldInputs.tenantAddress.value,
    tenantEmail: fieldInputs.tenantEmail.value,
    propertyAddress: fieldInputs.propertyAddress.value,
    periodStart: fieldInputs.periodStart.value,
    periodEnd: fieldInputs.periodEnd.value,
    rentAmount: fieldInputs.rentAmount.value,
    chargesAmount: fieldInputs.chargesAmount.value,
    paymentDate: fieldInputs.paymentDate.value,
  };
}
