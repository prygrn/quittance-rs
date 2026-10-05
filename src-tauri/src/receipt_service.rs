use quittance_core::{Receipt, validate_receipt};
use quittance_mailer::build_receipt_email;
use quittance_signature::load_signature;
use quittance_template::{IssueDetails, render_html};

use crate::app_config::{DeliveryConfig, ReceiptConfig};
use crate::command_error::{CommandError, CommandErrorCode};
use crate::config_source::ConfigSource;
use crate::delivery_factory::DeliveryFactory;
use crate::iso_date::parse_iso_date_field;
use crate::receipt_input_payload::ReceiptInputPayload;
use crate::sent_pdf_archive::SentPdfArchive;

/// Orchestration des commandes d'aperçu et d'envoi. Chaque appel relit la configuration,
/// pour qu'une configuration absente ou invalide soit signalée à l'UI par le code `config`.
pub struct ReceiptService<'a> {
    pub config_source: &'a dyn ConfigSource,
    pub delivery: &'a dyn DeliveryFactory,
    pub sent_pdf_archive: &'a dyn SentPdfArchive,
}

/// Quittance validée et son HTML, signature protégée comprise.
struct RenderedReceipt {
    receipt: Receipt,
    html: String,
}

impl ReceiptService<'_> {
    /// HTML complet de la quittance, signature protégée comprise, tel qu'il sera imprimé.
    /// `issue_date` (ISO `YYYY-MM-DD`) est la date locale de l'UI, portée par la mention
    /// « Fait à …, le … ». Seule la configuration de la quittance est exigée.
    pub fn preview(
        &self,
        template_id: &str,
        input: ReceiptInputPayload,
        issue_date: String,
    ) -> Result<String, CommandError> {
        let variables = self.config_source.variables()?;
        let config = ReceiptConfig::from_variables(&variables)?;
        Ok(self.render(&config, template_id, input, issue_date)?.html)
    }

    /// Régénère la quittance, l'imprime en PDF et l'envoie au locataire, avec copie cachée
    /// au bailleur. Le PDF est archivé avant l'envoi : un échec d'archivage n'envoie rien.
    pub fn send(
        &self,
        template_id: &str,
        input: ReceiptInputPayload,
        issue_date: String,
    ) -> Result<(), CommandError> {
        let variables = self.config_source.variables()?;
        let receipt_config = ReceiptConfig::from_variables(&variables)?;
        let delivery_config = DeliveryConfig::from_variables(&variables)?;
        let rendered = self.render(&receipt_config, template_id, input, issue_date)?;
        let pdf_renderer = self.delivery.pdf_renderer(delivery_config.chrome_path())?;
        let mailer = self.delivery.mailer(delivery_config.smtp())?;
        let pdf = pdf_renderer.render(&rendered.html)?;
        let email = build_receipt_email(&rendered.receipt, pdf)?;
        let attachment = email.attachment();
        self.sent_pdf_archive
            .store(attachment.file_name(), attachment.content())
            .map_err(|error| {
                CommandError::new(
                    CommandErrorCode::Unknown,
                    format!(
                        "sent PDF `{}` could not be archived: {error}",
                        attachment.file_name()
                    ),
                )
            })?;
        mailer.send(&email)?;
        Ok(())
    }

    fn render(
        &self,
        config: &ReceiptConfig,
        template_id: &str,
        input: ReceiptInputPayload,
        issue_date: String,
    ) -> Result<RenderedReceipt, CommandError> {
        let receipt = validate_receipt(config.landlord().clone(), input.into_receipt_input()?)?;
        let issue = IssueDetails {
            place: config.issue_place().to_owned(),
            date: parse_iso_date_field("issueDate", issue_date)?,
        };
        let signature = load_signature(config.signature_path())?.protect_for_receipt(&receipt);
        let html = render_html(template_id, &receipt, &issue, Some(signature.data_uri()))?;
        Ok(RenderedReceipt { receipt, html })
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::PathBuf;

    use quittance_core::{Party, validate_receipt};
    use quittance_mailer::{MailError, SmtpConfig, build_receipt_email};
    use quittance_pdf::PdfError;
    use quittance_signature::load_signature;
    use quittance_template::{IssueDetails, render_html};

    use super::*;
    use crate::app_config::{ConfigError, EnvFileProblem};
    use crate::test_support::{
        CHROME_PATH, DeliveryEvent, FAKE_PDF, FakeConfigSource, FakeDelivery, FakePdfArchive,
        STANDARD_TEMPLATE_ID, complete_variables, date, sample_payload,
    };

    /// Lendemain du paiement : distinct de toutes les dates de la saisie.
    const ISSUE_DATE: &str = "2026-10-06";
    const DELIVERY_VARIABLES: [&str; 5] = [
        "CHROME_PATH",
        "SMTP_HOST",
        "SMTP_PORT",
        "SMTP_USERNAME",
        "SMTP_PASSWORD",
    ];

    struct Harness {
        config_source: FakeConfigSource,
        delivery: FakeDelivery,
        archive_failure: Option<fn() -> io::Error>,
    }

    impl Harness {
        fn new() -> Self {
            Self {
                config_source: FakeConfigSource {
                    variables: complete_variables(),
                    failure: None,
                },
                delivery: FakeDelivery::default(),
                archive_failure: None,
            }
        }

        fn with_variable(mut self, name: &str, value: &str) -> Self {
            self.config_source
                .variables
                .insert(name.to_owned(), value.to_owned());
            self
        }

        fn without_variable(mut self, name: &str) -> Self {
            self.config_source.variables.remove(name);
            self
        }

        fn preview(&self, input: ReceiptInputPayload) -> Result<String, CommandError> {
            self.run(|service| service.preview(STANDARD_TEMPLATE_ID, input, ISSUE_DATE.to_owned()))
        }

        fn send(&self, input: ReceiptInputPayload) -> Result<(), CommandError> {
            self.run(|service| service.send(STANDARD_TEMPLATE_ID, input, ISSUE_DATE.to_owned()))
        }

        fn run<T>(&self, action: impl FnOnce(&ReceiptService) -> T) -> T {
            let archive = FakePdfArchive {
                journal: self.delivery.journal.clone(),
                failure: self.archive_failure,
            };
            action(&ReceiptService {
                config_source: &self.config_source,
                delivery: &self.delivery,
                sent_pdf_archive: &archive,
            })
        }

        fn events(&self) -> Vec<DeliveryEvent> {
            self.delivery.events()
        }

        fn sent_emails(&self) -> Vec<quittance_mailer::ReceiptEmail> {
            self.events()
                .into_iter()
                .filter_map(|event| match event {
                    DeliveryEvent::EmailSent(email) => Some(email),
                    _ => None,
                })
                .collect()
        }
    }

    /// Quittance attendue, rendue directement avec les crates de feature.
    fn expected_html() -> String {
        let landlord = Party::new(
            "Paul Durand",
            "3 avenue Foch, 69006 Lyon",
            "paul.durand@example.fr",
        )
        .unwrap();
        let receipt =
            validate_receipt(landlord, sample_payload().into_receipt_input().unwrap()).unwrap();
        let signature = load_signature(&crate::test_support::signature_fixture_path())
            .unwrap()
            .protect_for_receipt(&receipt);
        let issue = IssueDetails {
            place: "Lyon".to_owned(),
            date: date(2026, 10, 6),
        };
        render_html(
            STANDARD_TEMPLATE_ID,
            &receipt,
            &issue,
            Some(signature.data_uri()),
        )
        .unwrap()
    }

    fn assert_code<T: std::fmt::Debug>(result: Result<T, CommandError>, code: CommandErrorCode) {
        match result {
            Err(error) => assert_eq!(error.code(), code, "unexpected error {error:?}"),
            Ok(value) => panic!("expected a {code:?} error, got {value:?}"),
        }
    }

    // Aperçu

    #[test]
    fn given_valid_input_when_previewing_then_html_is_the_receipt_issued_on_ui_date_in_landlord_city()
     {
        let harness = Harness::new();

        let html = harness.preview(sample_payload()).unwrap();

        assert_eq!(html, expected_html());
    }

    #[test]
    fn given_valid_input_when_previewing_then_protected_signature_is_embedded() {
        let harness = Harness::new();

        let html = harness.preview(sample_payload()).unwrap();

        // Le template échappe la barre oblique de l'attribut `src` en `&#x2f;`.
        assert!(html.contains("src=\"data:image&#x2f;png;base64,"));
    }

    #[test]
    fn given_valid_input_when_previewing_then_nothing_is_rendered_archived_or_sent() {
        let harness = Harness::new();

        harness.preview(sample_payload()).unwrap();

        assert_eq!(harness.events(), []);
    }

    #[test]
    fn given_missing_configuration_when_previewing_then_code_is_config() {
        let harness = Harness::new().without_variable("LANDLORD_CITY");

        assert_code(harness.preview(sample_payload()), CommandErrorCode::Config);
    }

    #[test]
    fn given_malformed_date_when_previewing_then_code_is_validation() {
        let harness = Harness::new();
        let input = ReceiptInputPayload {
            period_end: "31/10/2026".to_owned(),
            ..sample_payload()
        };

        assert_code(harness.preview(input), CommandErrorCode::Validation);
    }

    #[test]
    fn given_input_refused_by_core_when_previewing_then_code_is_validation() {
        let harness = Harness::new();
        let input = ReceiptInputPayload {
            property_address: "  ".to_owned(),
            ..sample_payload()
        };

        assert_code(harness.preview(input), CommandErrorCode::Validation);
    }

    #[test]
    fn given_missing_signature_file_when_previewing_then_code_is_signature() {
        let harness =
            Harness::new().with_variable("SIGNATURE_PATH", "/nonexistent/quittance/signature.png");

        assert_code(
            harness.preview(sample_payload()),
            CommandErrorCode::Signature,
        );
    }

    #[test]
    fn given_unknown_template_when_previewing_then_code_is_template() {
        let harness = Harness::new();

        let result = harness
            .run(|service| service.preview("fancy", sample_payload(), ISSUE_DATE.to_owned()));

        assert_code(result, CommandErrorCode::Template);
    }

    #[test]
    fn given_issue_date_from_ui_when_previewing_then_mention_carries_that_date() {
        let harness = Harness::new();

        let html = harness
            .run(|service| {
                service.preview(
                    STANDARD_TEMPLATE_ID,
                    sample_payload(),
                    "2026-11-02".to_owned(),
                )
            })
            .unwrap();

        assert!(html.contains("02/11/2026"));
        assert!(!html.contains("06/10/2026"));
    }

    #[test]
    fn given_malformed_issue_date_when_previewing_then_code_is_validation() {
        let harness = Harness::new();

        let result = harness.run(|service| {
            service.preview(
                STANDARD_TEMPLATE_ID,
                sample_payload(),
                "06/10/2026".to_owned(),
            )
        });

        assert_code(result, CommandErrorCode::Validation);
    }

    #[test]
    fn given_no_chrome_nor_smtp_configuration_when_previewing_then_preview_is_rendered() {
        let harness = DELIVERY_VARIABLES
            .into_iter()
            .fold(Harness::new(), |harness, name| {
                harness.without_variable(name)
            });

        let html = harness.preview(sample_payload()).unwrap();

        assert_eq!(html, expected_html());
    }

    #[test]
    fn given_unreadable_configuration_source_when_previewing_then_code_is_config() {
        let mut harness = Harness::new();
        harness.config_source.failure = Some(|| ConfigError::UnreadableEnvFile {
            path: PathBuf::from("/opt/quittance/.env"),
            problem: EnvFileProblem::MalformedEntry {
                valid_entries_before: 3,
            },
        });

        assert_code(harness.preview(sample_payload()), CommandErrorCode::Config);
    }

    // Envoi

    #[test]
    fn given_valid_input_when_sending_then_receipt_is_printed_archived_then_emailed_in_order() {
        let harness = Harness::new();

        harness.send(sample_payload()).unwrap();

        let events = harness.events();
        let pdf_steps: Vec<&str> = events
            .iter()
            .filter_map(|event| match event {
                DeliveryEvent::PdfRendered(_) => Some("rendered"),
                DeliveryEvent::PdfArchived { .. } => Some("archived"),
                DeliveryEvent::EmailSent(_) => Some("sent"),
                _ => None,
            })
            .collect();
        assert_eq!(pdf_steps, ["rendered", "archived", "sent"]);
    }

    #[test]
    fn given_valid_input_when_sending_then_pdf_renderer_receives_the_previewed_html() {
        let harness = Harness::new();

        harness.send(sample_payload()).unwrap();

        assert!(
            harness
                .events()
                .contains(&DeliveryEvent::PdfRendered(expected_html()))
        );
    }

    #[test]
    fn given_valid_input_when_sending_then_delivery_uses_the_configured_chrome_and_smtp() {
        let harness = Harness::new();

        harness.send(sample_payload()).unwrap();

        let events = harness.events();
        assert!(events.contains(&DeliveryEvent::RendererCreated(PathBuf::from(CHROME_PATH))));
        let smtp = SmtpConfig::from_variables(&complete_variables()).unwrap();
        assert!(events.contains(&DeliveryEvent::MailerCreated(smtp)));
    }

    #[test]
    fn given_valid_input_when_sending_then_tenant_receives_the_pdf_with_landlord_in_blind_copy() {
        let harness = Harness::new();

        harness.send(sample_payload()).unwrap();

        let landlord = Party::new(
            "Paul Durand",
            "3 avenue Foch, 69006 Lyon",
            "paul.durand@example.fr",
        )
        .unwrap();
        let receipt =
            validate_receipt(landlord, sample_payload().into_receipt_input().unwrap()).unwrap();
        let expected_email = build_receipt_email(&receipt, FAKE_PDF.to_vec()).unwrap();
        assert_eq!(harness.sent_emails(), [expected_email]);
        let email = &harness.sent_emails()[0];
        assert_eq!(email.recipient().email(), "jeanne.martin@example.fr");
        assert_eq!(email.blind_copy(), "paul.durand@example.fr");
    }

    #[test]
    fn given_valid_input_when_sending_then_pdf_is_archived_under_its_attachment_name() {
        let harness = Harness::new();

        harness.send(sample_payload()).unwrap();

        assert!(harness.events().contains(&DeliveryEvent::PdfArchived {
            file_name: "quittance-2026-10.pdf".to_owned(),
            pdf: FAKE_PDF.to_vec(),
        }));
    }

    #[test]
    fn given_missing_configuration_when_sending_then_code_is_config_and_nothing_happens() {
        let harness = Harness::new().without_variable("SMTP_HOST");

        assert_code(harness.send(sample_payload()), CommandErrorCode::Config);
        assert_eq!(harness.events(), []);
    }

    #[test]
    fn given_each_missing_delivery_variable_when_sending_then_code_is_config_and_nothing_happens() {
        for name in DELIVERY_VARIABLES {
            let harness = Harness::new().without_variable(name);

            assert_code(harness.send(sample_payload()), CommandErrorCode::Config);
            assert_eq!(harness.events(), [], "without {name}");
        }
    }

    #[test]
    fn given_malformed_issue_date_when_sending_then_code_is_validation_and_nothing_happens() {
        let harness = Harness::new();

        let result = harness.run(|service| {
            service.send(
                STANDARD_TEMPLATE_ID,
                sample_payload(),
                "2026-10-32".to_owned(),
            )
        });

        assert_code(result, CommandErrorCode::Validation);
        assert_eq!(harness.events(), []);
    }

    #[test]
    fn given_invalid_input_when_sending_then_code_is_validation_and_nothing_is_rendered() {
        let harness = Harness::new();
        let input = ReceiptInputPayload {
            tenant_email: "jeanne".to_owned(),
            ..sample_payload()
        };

        assert_code(harness.send(input), CommandErrorCode::Validation);
        assert!(
            !harness
                .events()
                .iter()
                .any(|event| matches!(event, DeliveryEvent::PdfRendered(_)))
        );
    }

    #[test]
    fn given_missing_signature_file_when_sending_then_code_is_signature_and_nothing_is_sent() {
        let harness =
            Harness::new().with_variable("SIGNATURE_PATH", "/nonexistent/quittance/signature.png");

        assert_code(harness.send(sample_payload()), CommandErrorCode::Signature);
        assert_eq!(harness.sent_emails(), []);
    }

    #[test]
    fn given_chrome_not_found_when_sending_then_code_is_pdf_and_nothing_is_sent() {
        let mut harness = Harness::new();
        harness.delivery.renderer_creation_failure = Some(|| PdfError::ChromeNotFound {
            path: PathBuf::from(CHROME_PATH),
        });

        assert_code(harness.send(sample_payload()), CommandErrorCode::Pdf);
        assert_eq!(harness.sent_emails(), []);
    }

    #[test]
    fn given_pdf_rendering_failure_when_sending_then_code_is_pdf_and_nothing_is_archived_or_sent() {
        let mut harness = Harness::new();
        harness.delivery.rendering_failure = Some(|| PdfError::Rendering("chrome crashed".into()));

        assert_code(harness.send(sample_payload()), CommandErrorCode::Pdf);
        assert!(!harness.events().iter().any(|event| matches!(
            event,
            DeliveryEvent::PdfArchived { .. } | DeliveryEvent::EmailSent(_)
        )));
    }

    #[test]
    fn given_empty_pdf_when_sending_then_code_is_mail_and_nothing_is_sent() {
        let mut harness = Harness::new();
        harness.delivery.pdf = Some(Vec::new());

        assert_code(harness.send(sample_payload()), CommandErrorCode::Mail);
        assert_eq!(harness.sent_emails(), []);
    }

    #[test]
    fn given_mailer_setup_failure_when_sending_then_code_is_mail_and_nothing_is_sent() {
        let mut harness = Harness::new();
        harness.delivery.mailer_creation_failure =
            Some(|| MailError::TransportSetup("no transport".into()));

        assert_code(harness.send(sample_payload()), CommandErrorCode::Mail);
        assert_eq!(harness.sent_emails(), []);
    }

    #[test]
    fn given_delivery_failure_when_sending_then_code_is_mail() {
        let mut harness = Harness::new();
        harness.delivery.sending_failure =
            Some(|| MailError::Delivery("connection refused".into()));

        assert_code(harness.send(sample_payload()), CommandErrorCode::Mail);
    }

    #[test]
    fn given_archive_failure_when_sending_then_code_is_unknown_and_nothing_is_sent() {
        let mut harness = Harness::new();
        harness.archive_failure = Some(|| io::Error::from(io::ErrorKind::PermissionDenied));

        assert_code(harness.send(sample_payload()), CommandErrorCode::Unknown);
        assert_eq!(harness.sent_emails(), []);
    }

    #[test]
    fn given_unknown_template_when_sending_then_code_is_template_and_nothing_is_rendered() {
        let harness = Harness::new();

        let result =
            harness.run(|service| service.send("fancy", sample_payload(), ISSUE_DATE.to_owned()));

        assert_code(result, CommandErrorCode::Template);
        assert!(
            !harness
                .events()
                .iter()
                .any(|event| matches!(event, DeliveryEvent::PdfRendered(_)))
        );
    }
}
