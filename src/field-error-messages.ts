import type { FieldErrorCode } from "./receipt-form";

/** Message affiché sous un champ pour chaque erreur de saisie, repris de la maquette. */
export const FIELD_ERROR_MESSAGES: Readonly<Record<FieldErrorCode, string>> = {
  missing: "Ce champ est obligatoire.",
  invalidEmail: "Adresse email invalide. Exemple : nom@domaine.fr",
  invalidDate: "Date invalide. Format attendu : jj/mm/aaaa",
  invertedPeriod: "La fin de période ne peut pas précéder le début.",
  invalidAmount: "Montant invalide. Exemple : 650 ou 1 234,56",
  negativeAmount: "Le montant ne peut pas être négatif.",
  tooManyDecimals: "Deux décimales au maximum.",
  // La maquette annonçait un plafond de 100 000 € ; la limite réelle est celle de `parseAmountCents`.
  amountTooLarge: "Montant trop élevé.",
};
