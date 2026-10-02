use crate::{MailError, ReceiptEmail};

/// Envoi de l'email de quittance, derrière un trait pour tester les appelants avec des fakes.
pub trait Mailer {
    fn send(&self, email: &ReceiptEmail) -> Result<(), MailError>;
}
