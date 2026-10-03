use std::error::Error as StdError;

use thiserror::Error;

/// Cause d'échec boxée : laisse les fakes de [`crate::Mailer`] simuler un échec
/// sans dépendre des types d'erreur de lettre, dont les constructeurs sont privés.
pub type MailFailureSource = Box<dyn StdError + Send + Sync + 'static>;

/// Erreur de configuration SMTP, de construction ou d'envoi de l'email de quittance.
#[derive(Debug, Error)]
pub enum MailError {
    #[error("environment variable `{0}` is missing or blank")]
    MissingVariable(&'static str),
    #[error("SMTP port `{0}` is not a port number between 1 and 65535")]
    InvalidPort(String),
    #[error("SMTP security mode `{0}` is unknown, expected starttls, tls or none")]
    UnknownSecurityMode(String),
    #[error("SMTP security mode none is only allowed towards a loopback host, not `{0}`")]
    UnencryptedRemoteHost(String),
    #[error("receipt PDF is empty")]
    EmptyAttachment,
    #[error("SMTP transport could not be set up: {0}")]
    TransportSetup(#[source] MailFailureSource),
    #[error("receipt email could not be assembled: {0}")]
    InvalidMessage(#[source] MailFailureSource),
    #[error("receipt email could not be delivered: {0}")]
    Delivery(#[source] MailFailureSource),
}
