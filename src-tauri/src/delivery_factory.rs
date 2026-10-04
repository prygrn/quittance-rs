use std::path::Path;

use quittance_mailer::{MailError, Mailer, SmtpConfig};
use quittance_pdf::{PdfError, PdfRenderer};

/// Crée, à partir de la configuration chargée, les effets de bord de l'envoi : rendu PDF
/// et envoi de l'email. Derrière un trait pour tester l'envoi avec des fakes.
pub trait DeliveryFactory {
    fn pdf_renderer(&self, chrome_path: &Path) -> Result<Box<dyn PdfRenderer>, PdfError>;
    fn mailer(&self, smtp: &SmtpConfig) -> Result<Box<dyn Mailer>, MailError>;
}

/// Rendu par Chromium headless et envoi par SMTP.
pub struct ChromiumSmtpDelivery;

impl DeliveryFactory for ChromiumSmtpDelivery {
    fn pdf_renderer(&self, chrome_path: &Path) -> Result<Box<dyn PdfRenderer>, PdfError> {
        let _ = chrome_path;
        todo!()
    }

    fn mailer(&self, smtp: &SmtpConfig) -> Result<Box<dyn Mailer>, MailError> {
        let _ = smtp;
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::complete_variables;

    #[test]
    fn given_missing_chrome_binary_when_creating_renderer_then_chrome_is_not_found() {
        let result = ChromiumSmtpDelivery.pdf_renderer(Path::new("/nonexistent/chrome"));

        assert!(matches!(result, Err(PdfError::ChromeNotFound { .. })));
    }

    #[test]
    fn given_existing_chrome_path_when_creating_renderer_then_renderer_is_created() {
        let existing_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");

        let result = ChromiumSmtpDelivery.pdf_renderer(&existing_file);

        assert!(result.is_ok());
    }

    #[test]
    fn given_valid_smtp_config_when_creating_mailer_then_mailer_is_created() {
        let smtp = SmtpConfig::from_variables(&complete_variables()).unwrap();

        let result = ChromiumSmtpDelivery.mailer(&smtp);

        assert!(result.is_ok());
    }
}
