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

// Identifiants des champs dans index.html.
const FIELD_IDS: Readonly<Record<keyof ReceiptFormEntries, string>> = {
  tenantName: "tenant-name",
  tenantAddress: "tenant-address",
  tenantEmail: "tenant-email",
  propertyAddress: "property-address",
  periodStart: "period-start",
  periodEnd: "period-end",
  rentAmount: "rent-amount",
  chargesAmount: "charges-amount",
  paymentDate: "payment-date",
};

/** Tape chaque valeur dans son champ, dans l'ordre du formulaire. */
export async function fillReceiptForm(entries: ReceiptFormEntries): Promise<void> {
  const fields = Object.keys(FIELD_IDS) as (keyof ReceiptFormEntries)[];
  await fields.reduce(
    (previous: Promise<void>, field) =>
      previous.then(() => $(`#${FIELD_IDS[field]}`).setValue(entries[field])),
    Promise.resolve(),
  );
}
