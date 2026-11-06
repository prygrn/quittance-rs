import type { ReceiptFormEntries } from "./fill-receipt-form";
import { RECEIPT_FORM_FIELD_IDS } from "./receipt-form-field-ids";

function readField(field: keyof ReceiptFormEntries): Promise<string> {
  return $(`#${RECEIPT_FORM_FIELD_IDS[field]}`).getValue();
}

/** Relit la valeur affichée par chaque champ du formulaire. */
export async function readReceiptForm(): Promise<ReceiptFormEntries> {
  return {
    tenantName: await readField("tenantName"),
    tenantAddress: await readField("tenantAddress"),
    tenantEmail: await readField("tenantEmail"),
    propertyAddress: await readField("propertyAddress"),
    periodStart: await readField("periodStart"),
    periodEnd: await readField("periodEnd"),
    rentAmount: await readField("rentAmount"),
    chargesAmount: await readField("chargesAmount"),
    paymentDate: await readField("paymentDate"),
  };
}
