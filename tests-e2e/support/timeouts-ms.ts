/** Délais d'attente des scénarios, en millisecondes. */
export const TIMEOUTS_MS = {
  /** Un scénario complet, de l'ouverture de l'app à la relecture de Mailpit. */
  scenario: 120_000,
  /** Chargement de la liste des modèles au démarrage de l'app. */
  templates: 15_000,
  /** Affichage d'une erreur de saisie après un clic. */
  fieldError: 5_000,
  /** Rendu de l'aperçu par le backend. */
  preview: 30_000,
  /** Envoi de la quittance, impression du PDF comprise. */
  send: 60_000,
  /** Arrivée du message dans Mailpit après un envoi confirmé. */
  delivery: 10_000,
} as const;
