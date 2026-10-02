use lettre::{Message, SmtpTransport};

use crate::{MailError, Mailer, ReceiptEmail, SmtpConfig};

/// Envoi réel par SMTP via lettre.
#[derive(Clone)]
pub struct SmtpMailer {
    transport: SmtpTransport,
}

impl SmtpMailer {
    pub fn new(config: &SmtpConfig) -> Result<Self, MailError> {
        todo!()
    }
}

impl Mailer for SmtpMailer {
    fn send(&self, email: &ReceiptEmail) -> Result<(), MailError> {
        todo!()
    }
}

fn to_lettre_message(email: &ReceiptEmail) -> Result<Message, MailError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use quittance_core::{Party, ReceiptInput, validate_receipt};
    use time::macros::date;

    use super::*;
    use crate::build_receipt_email;
    use crate::config::variable_names;

    const LANDLORD_EMAIL: &str = "paul.durand@example.fr";
    const TENANT_EMAIL: &str = "jeanne.martin@example.fr";

    fn sample_email() -> ReceiptEmail {
        let landlord =
            Party::new("Paul Durand", "3 avenue Foch, 69006 Lyon", LANDLORD_EMAIL).unwrap();
        let input = ReceiptInput {
            tenant_name: "Jeanne Martin".to_owned(),
            tenant_address: "12 rue des Lilas, 75011 Paris".to_owned(),
            tenant_email: TENANT_EMAIL.to_owned(),
            property_address: "12 rue des Lilas, 75011 Paris".to_owned(),
            period_start: date!(2026 - 10 - 01),
            period_end: date!(2026 - 10 - 31),
            rent_cents: 85_000,
            charges_cents: 5_000,
            payment_date: date!(2026 - 10 - 05),
        };
        let receipt = validate_receipt(landlord, input).unwrap();
        build_receipt_email(&receipt, b"%PDF-1.7 quittance".to_vec()).unwrap()
    }

    fn config_with_security(security: &str) -> SmtpConfig {
        let variables = HashMap::from([
            (
                variable_names::HOST.to_owned(),
                "smtp.example.fr".to_owned(),
            ),
            (variable_names::PORT.to_owned(), "587".to_owned()),
            (
                variable_names::USERNAME.to_owned(),
                LANDLORD_EMAIL.to_owned(),
            ),
            (
                variable_names::PASSWORD.to_owned(),
                "s3cr3t-value".to_owned(),
            ),
            (variable_names::SECURITY.to_owned(), security.to_owned()),
        ]);
        SmtpConfig::from_variables(&variables).unwrap()
    }

    #[test]
    fn given_each_security_mode_when_creating_mailer_then_transport_is_ready() {
        for security in ["starttls", "tls", "none"] {
            let config = config_with_security(security);

            let result = SmtpMailer::new(&config);

            assert!(result.is_ok(), "{security} should yield a mailer");
        }
    }

    #[test]
    fn given_receipt_email_when_converting_then_envelope_targets_tenant_and_landlord() {
        let email = sample_email();

        let message = to_lettre_message(&email).unwrap();

        let recipients: Vec<&str> = message
            .envelope()
            .to()
            .iter()
            .map(|address| address.as_ref())
            .collect();
        assert_eq!(recipients, [TENANT_EMAIL, LANDLORD_EMAIL]);
        assert_eq!(
            message.envelope().from().map(|address| address.as_ref()),
            Some(LANDLORD_EMAIL)
        );
    }

    #[test]
    fn given_receipt_email_when_converting_then_blind_copy_is_absent_from_headers() {
        let email = sample_email();

        let message = to_lettre_message(&email).unwrap();

        let formatted = String::from_utf8_lossy(&message.formatted()).to_lowercase();
        assert!(!formatted.contains("bcc:"));
    }
}
