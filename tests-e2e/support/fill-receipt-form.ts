import { RECEIPT_FORM_FIELD_IDS } from "./receipt-form-field-ids";

/** Saisie du formulaire telle qu'un utilisateur la tape : dates en jj/mm/aaaa, montants en euros. */
export interface ReceiptFormEntries {
  readonly tenantName: string;
  readonly tenantAddress: string;
  readonly tenantEmail: string;
  readonly propertyAddress: string;
  readonly periodStart: string;
  readonly periodEnd: string;
  readonly rentAmount: string;
  readonly chargesAmount: string;
  readonly paymentDate: string;
}

/** Tape chaque valeur fournie dans son champ, dans l'ordre du formulaire, sans toucher aux autres. */
export async function fillReceiptForm(entries: Partial<ReceiptFormEntries>): Promise<void> {
  const fields = Object.keys(RECEIPT_FORM_FIELD_IDS) as (keyof ReceiptFormEntries)[];
  await fields.reduce((previous: Promise<void>, field) => {
    const value = entries[field];
    return value === undefined
      ? previous
      : previous.then(() => $(`#${RECEIPT_FORM_FIELD_IDS[field]}`).setValue(value));
  }, Promise.resolve());
}
