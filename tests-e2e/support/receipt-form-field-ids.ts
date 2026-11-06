import type { ReceiptFormEntries } from "./fill-receipt-form";

/** Identifiants des champs du formulaire dans index.html, dans l'ordre du formulaire. */
export const RECEIPT_FORM_FIELD_IDS: Readonly<Record<keyof ReceiptFormEntries, string>> = {
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
