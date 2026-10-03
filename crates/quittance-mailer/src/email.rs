use std::fmt;

use quittance_core::{Date, Party, Receipt};

use crate::MailError;

/// Libellés fixes de l'email, en français comme la quittance.
mod wording {
    pub const SUBJECT_PREFIX: &str = "Quittance de loyer –";
    pub const FILE_NAME_PREFIX: &str = "quittance";
    pub const PDF_CONTENT_TYPE: &str = "application/pdf";
    /// Indexé par numéro de mois moins un.
    pub const FRENCH_MONTH_NAMES: [&str; 12] = [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ];
}

/// Nom et adresse d'un correspondant de l'email.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailContact {
    name: String,
    email: String,
}

impl EmailContact {
    fn from_party(party: &Party) -> Self {
        Self {
            name: party.name().to_owned(),
            email: party.email().to_owned(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}

/// Quittance PDF jointe à l'email.
#[derive(Clone, PartialEq, Eq)]
pub struct PdfAttachment {
    file_name: String,
    content: Vec<u8>,
}

/// Les octets du PDF n'encombrent pas les traces : seul le nom du fichier apparaît.
impl fmt::Debug for PdfAttachment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PdfAttachment")
            .field("file_name", &self.file_name)
            .finish_non_exhaustive()
    }
}

impl PdfAttachment {
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn content_type(&self) -> &str {
        wording::PDF_CONTENT_TYPE
    }

    pub fn content(&self) -> &[u8] {
        &self.content
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
        &self.sender
    }

    pub fn recipient(&self) -> &EmailContact {
        &self.recipient
    }

    /// Adresse en copie cachée : celle du bailleur, dont la copie sert d'archive.
    pub fn blind_copy(&self) -> &str {
        &self.blind_copy
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn attachment(&self) -> &PdfAttachment {
        &self.attachment
    }
}

/// Construit l'email envoyé par le bailleur au locataire, quittance PDF en pièce jointe.
/// La période est désignée par le mois de son début.
pub fn build_receipt_email(receipt: &Receipt, pdf: Vec<u8>) -> Result<ReceiptEmail, MailError> {
    if pdf.is_empty() {
        return Err(MailError::EmptyAttachment);
    }
    let landlord = receipt.landlord();
    let tenant = receipt.tenant();
    let period_start = receipt.period().start();
    let period_label = french_month_and_year(period_start);
    Ok(ReceiptEmail {
        sender: EmailContact::from_party(landlord),
        recipient: EmailContact::from_party(tenant),
        blind_copy: landlord.email().to_owned(),
        subject: format!("{} {period_label}", wording::SUBJECT_PREFIX),
        body: format!(
            "Bonjour {},\n\n\
             Veuillez trouver ci-joint votre quittance de loyer pour {period_label}.\n\n\
             Cordialement,\n\
             {}\n",
            tenant.name(),
            landlord.name()
        ),
        attachment: PdfAttachment {
            file_name: format!(
                "{}-{}-{:02}.pdf",
                wording::FILE_NAME_PREFIX,
                period_start.year(),
                month_number(period_start)
            ),
            content: pdf,
        },
    })
}

/// Par exemple « octobre 2026 ».
fn french_month_and_year(date: Date) -> String {
    let month_name = wording::FRENCH_MONTH_NAMES[usize::from(month_number(date)) - 1];
    format!("{month_name} {}", date.year())
}

/// Numéro du mois, de 1 à 12.
fn month_number(date: Date) -> u8 {
    u8::from(date.month())
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
    fn given_receipt_email_when_debug_formatting_then_pdf_bytes_are_hidden() {
        let email = build_october_email();
        let pdf_bytes = format!("{PDF_CONTENT:?}");

        let debug_output = format!("{email:?}");

        assert!(!debug_output.contains(&pdf_bytes));
        assert!(debug_output.contains("quittance-2026-10.pdf"));
    }

    #[test]
    fn given_pdf_attachment_when_debug_formatting_then_only_file_name_is_shown() {
        let email = build_october_email();
        let pdf_bytes = format!("{PDF_CONTENT:?}");

        let debug_output = format!("{:?}", email.attachment());

        assert!(!debug_output.contains(&pdf_bytes));
        assert!(debug_output.contains("quittance-2026-10.pdf"));
    }

    #[test]
    fn given_empty_pdf_when_building_email_then_attachment_is_rejected() {
        let receipt = sample_october_receipt();

        let result = build_receipt_email(&receipt, Vec::new());

        assert!(matches!(result, Err(MailError::EmptyAttachment)));
    }
}
