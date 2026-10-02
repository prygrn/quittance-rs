//! Construction de l'email de quittance et envoi par SMTP.
//!
//! [`build_receipt_email`] est pure ; l'envoi passe par le trait [`Mailer`],
//! implémenté par [`SmtpMailer`] à partir d'une [`SmtpConfig`] validée.

// Temporaire : API en stubs `todo!()` le temps d'écrire les tests, retiré à l'implémentation.
#![allow(unused_variables, dead_code)]

mod config;
mod email;
mod error;
mod mailer;
mod smtp_mailer;

pub use config::SmtpConfig;
pub use email::{EmailContact, PdfAttachment, ReceiptEmail, build_receipt_email};
pub use error::{MailError, MailFailureSource};
pub use mailer::Mailer;
pub use smtp_mailer::SmtpMailer;

#[cfg(test)]
mod tests {
    use quittance_core::Receipt;

    use super::*;

    fn assert_is_mailer<T: Mailer>() {}

    /// Les appelants manipulent un `&dyn Mailer` pour injecter des fakes.
    fn assert_is_object_safe(_: Option<&dyn Mailer>) {}

    #[test]
    fn given_crate_when_using_public_api_then_contract_items_are_exposed() {
        let building: fn(&Receipt, Vec<u8>) -> Result<ReceiptEmail, MailError> =
            build_receipt_email;
        let loading: fn(
            &std::collections::HashMap<String, String>,
        ) -> Result<SmtpConfig, MailError> = SmtpConfig::from_variables;
        let creating: fn(&SmtpConfig) -> Result<SmtpMailer, MailError> = SmtpMailer::new;

        assert_is_mailer::<SmtpMailer>();
        assert_is_object_safe(None);
        let _ = (building, loading, creating);
    }
}
