use quittance_core::Receipt;

use crate::MailError;

/// Nom et adresse d'un correspondant de l'email.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailContact {
    name: String,
    email: String,
}

impl EmailContact {
    pub fn name(&self) -> &str {
        todo!()
    }

    pub fn email(&self) -> &str {
        todo!()
    }
}

/// Quittance PDF jointe à l'email.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfAttachment {
    file_name: String,
    content: Vec<u8>,
}

impl PdfAttachment {
    pub fn file_name(&self) -> &str {
        todo!()
    }

    pub fn content_type(&self) -> &str {
        todo!()
    }

    pub fn content(&self) -> &[u8] {
        todo!()
    }
}

/// Email de quittance prêt à l'envoi, indépendant du transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptEmail {
    sender: EmailContact,
    recipient: EmailContact,
    blind_copy: String,
    subject: String,
    body: String,
    attachment: PdfAttachment,
}

impl ReceiptEmail {
    pub fn sender(&self) -> &EmailContact {
        todo!()
    }

    pub fn recipient(&self) -> &EmailContact {
        todo!()
    }

    /// Adresse en copie cachée : celle du bailleur, dont la copie sert d'archive.
    pub fn blind_copy(&self) -> &str {
        todo!()
    }

    pub fn subject(&self) -> &str {
        todo!()
    }

    pub fn body(&self) -> &str {
        todo!()
    }

    pub fn attachment(&self) -> &PdfAttachment {
        todo!()
    }
}

/// Construit l'email envoyé par le bailleur au locataire, quittance PDF en pièce jointe.
pub fn build_receipt_email(receipt: &Receipt, pdf: Vec<u8>) -> Result<ReceiptEmail, MailError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use quittance_core::{Date, Party, ReceiptInput, validate_receipt};
    use time::macros::date;

    use super::*;

    const LANDLORD_NAME: &str = "Paul Durand";
    const LANDLORD_EMAIL: &str = "paul.durand@example.fr";
    const TENANT_NAME: &str = "Jeanne Martin";
    const TENANT_EMAIL: &str = "jeanne.martin@example.fr";
    const PDF_CONTENT: &[u8] = b"%PDF-1.7 quittance";

    fn sample_receipt_for_period(period_start: Date, period_end: Date) -> Receipt {
        let landlord =
            Party::new(LANDLORD_NAME, "3 avenue Foch, 69006 Lyon", LANDLORD_EMAIL).unwrap();
        let input = ReceiptInput {
            tenant_name: TENANT_NAME.to_owned(),
            tenant_address: "12 rue des Lilas, 75011 Paris".to_owned(),
            tenant_email: TENANT_EMAIL.to_owned(),
            property_address: "12 rue des Lilas, 75011 Paris".to_owned(),
            period_start,
            period_end,
            rent_cents: 85_000,
            charges_cents: 5_000,
            payment_date: period_start,
        };
        validate_receipt(landlord, input).unwrap()
    }

    fn sample_october_receipt() -> Receipt {
        sample_receipt_for_period(date!(2026 - 10 - 01), date!(2026 - 10 - 31))
    }

    fn build_october_email() -> ReceiptEmail {
        build_receipt_email(&sample_october_receipt(), PDF_CONTENT.to_vec()).unwrap()
    }

    #[test]
    fn given_receipt_when_building_email_then_tenant_is_recipient() {
        let email = build_october_email();

        assert_eq!(email.recipient().name(), TENANT_NAME);
        assert_eq!(email.recipient().email(), TENANT_EMAIL);
    }

    #[test]
    fn given_receipt_when_building_email_then_landlord_is_sender() {
        let email = build_october_email();

        assert_eq!(email.sender().name(), LANDLORD_NAME);
        assert_eq!(email.sender().email(), LANDLORD_EMAIL);
    }

    #[test]
    fn given_receipt_when_building_email_then_landlord_is_blind_copied() {
        let email = build_october_email();

        assert_eq!(email.blind_copy(), LANDLORD_EMAIL);
    }

    #[test]
    fn given_october_receipt_when_building_email_then_subject_names_the_period_in_french() {
        let email = build_october_email();

        assert_eq!(email.subject(), "Quittance de loyer – octobre 2026");
    }

    #[test]
    fn given_receipt_of_each_month_when_building_email_then_subject_uses_french_month_name() {
        let french_month_names = [
            (date!(2027 - 01 - 01), "janvier"),
            (date!(2027 - 02 - 01), "février"),
            (date!(2027 - 03 - 01), "mars"),
            (date!(2027 - 04 - 01), "avril"),
            (date!(2027 - 05 - 01), "mai"),
            (date!(2027 - 06 - 01), "juin"),
            (date!(2027 - 07 - 01), "juillet"),
            (date!(2027 - 08 - 01), "août"),
            (date!(2027 - 09 - 01), "septembre"),
            (date!(2027 - 10 - 01), "octobre"),
            (date!(2027 - 11 - 01), "novembre"),
            (date!(2027 - 12 - 01), "décembre"),
        ];

        for (period_start, month_name) in french_month_names {
            let receipt = sample_receipt_for_period(period_start, period_start);

            let email = build_receipt_email(&receipt, PDF_CONTENT.to_vec()).unwrap();

            assert_eq!(
                email.subject(),
                format!("Quittance de loyer – {month_name} 2027")
            );
        }
    }

    #[test]
    fn given_period_spanning_two_months_when_building_email_then_subject_uses_starting_month() {
        let receipt = sample_receipt_for_period(date!(2026 - 12 - 15), date!(2027 - 01 - 14));

        let email = build_receipt_email(&receipt, PDF_CONTENT.to_vec()).unwrap();

        assert_eq!(email.subject(), "Quittance de loyer – décembre 2026");
    }

    #[test]
    fn given_receipt_when_building_email_then_body_greets_tenant_and_names_the_period() {
        let email = build_october_email();

        assert!(email.body().contains(TENANT_NAME));
        assert!(email.body().contains("octobre 2026"));
        assert!(email.body().contains(LANDLORD_NAME));
    }

    #[test]
    fn given_receipt_when_building_email_then_pdf_is_attached_under_period_file_name() {
        let email = build_october_email();

        assert_eq!(email.attachment().file_name(), "quittance-2026-10.pdf");
        assert_eq!(email.attachment().content_type(), "application/pdf");
        assert_eq!(email.attachment().content(), PDF_CONTENT);
    }

    #[test]
    fn given_single_digit_month_when_building_email_then_file_name_month_is_zero_padded() {
        let receipt = sample_receipt_for_period(date!(2026 - 03 - 01), date!(2026 - 03 - 31));

        let email = build_receipt_email(&receipt, PDF_CONTENT.to_vec()).unwrap();

        assert_eq!(email.attachment().file_name(), "quittance-2026-03.pdf");
    }

    #[test]
    fn given_empty_pdf_when_building_email_then_attachment_is_rejected() {
        let receipt = sample_october_receipt();

        let result = build_receipt_email(&receipt, Vec::new());

        assert!(matches!(result, Err(MailError::EmptyAttachment)));
    }
}
