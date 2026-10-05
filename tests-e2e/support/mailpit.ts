/** Mailpit local qui reçoit les envois des scénarios : SMTP pour l'app, API pour les vérifications. */
export const MAILPIT = {
  host: "127.0.0.1",
  smtpPort: 1025,
  apiPort: 8025,
} as const;
