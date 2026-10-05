/**
 * Quittance d'octobre 2026 saisie dans les scénarios, avec les valeurs que l'app doit
 * en tirer : montants et dates au format français de la quittance, pièce jointe nommée
 * d'après la période.
 */
export const SAMPLE_RECEIPT = {
  entries: {
    tenantName: "Jeanne Martin",
    tenantAddress: "8 rue Victor Hugo, 75011 Paris",
    tenantEmail: "jeanne.martin@example.fr",
    propertyAddress: "12 rue des Lilas, 75011 Paris",
    periodStart: "01/10/2026",
    periodEnd: "31/10/2026",
    rentAmount: "650",
    chargesAmount: "50,50",
    paymentDate: "05/10/2026",
  },
  // Espace insécable (U+00A0) avant le symbole, comme sur la quittance.
  expectedRent: "650,00 €",
  expectedCharges: "50,50 €",
  expectedTotal: "700,50 €",
  expectedAttachmentName: "quittance-2026-10.pdf",
} as const;
