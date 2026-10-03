import type { ReceiptInput } from "./api";
import type { ReceiptFormValues } from "./receipt-form";

// Données partagées par les tests : une saisie valide, brute puis typée.

export function sampleReceiptFormValues(): ReceiptFormValues {
  return {
    tenantName: "Jeanne Martin",
    tenantAddress: "12 rue des Lilas, 75011 Paris",
    tenantEmail: "jeanne.martin@example.fr",
    propertyAddress: "12 rue des Lilas, 75011 Paris",
    periodStart: "2026-10-01",
    periodEnd: "2026-10-31",
    rentAmount: "650",
    chargesAmount: "50,50",
    paymentDate: "2026-10-05",
  };
}

export function sampleReceiptInput(): ReceiptInput {
  return {
    tenantName: "Jeanne Martin",
    tenantAddress: "12 rue des Lilas, 75011 Paris",
    tenantEmail: "jeanne.martin@example.fr",
    propertyAddress: "12 rue des Lilas, 75011 Paris",
    periodStart: "2026-10-01",
    periodEnd: "2026-10-31",
    rentCents: 65_000,
    chargesCents: 5_050,
    paymentDate: "2026-10-05",
  };
}
