use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use quittance_core::Date;
use quittance_mailer::{MailError, Mailer, ReceiptEmail, SmtpConfig};
use quittance_pdf::{PdfError, PdfRenderer};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tauri::ipc::{InvokeResponseBody, IpcResponse};

use crate::app_config::ConfigError;
use crate::config_source::ConfigSource;
use crate::delivery_factory::DeliveryFactory;
use crate::receipt_input_payload::ReceiptInputPayload;
use crate::sent_pdf_archive::SentPdfArchive;

pub(crate) const STANDARD_TEMPLATE_ID: &str = "standard";
pub(crate) const CHROME_PATH: &str = "/opt/chromium/chrome";
pub(crate) const FAKE_PDF: &[u8] = b"%PDF-1.7 fake receipt";

pub(crate) fn date(year: i32, month: u8, day: u8) -> Date {
    Date::from_calendar_date(year, month.try_into().unwrap(), day).unwrap()
}

/// JSON produit par Tauri pour une valeur renvoyée à l'UI.
pub(crate) fn to_ipc_json<T: Serialize>(value: T) -> String {
    match value.body().unwrap() {
        InvokeResponseBody::Json(json) => json,
        InvokeResponseBody::Raw(_) => panic!("a serializable value is sent as JSON"),
    }
}

/// Lecture d'un JSON avec le désérialiseur qu'utilise Tauri.
pub(crate) fn from_ipc_json<T: DeserializeOwned>(json: &str) -> Result<T, impl Debug> {
    InvokeResponseBody::Json(json.to_owned()).deserialize::<T>()
}

/// PNG d'une signature fictive, versionné avec les tests.
pub(crate) fn signature_fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/signature.png")
}

pub(crate) fn complete_variables() -> HashMap<String, String> {
    let signature_path = signature_fixture_path().to_string_lossy().into_owned();
    [
        ("LANDLORD_NAME", "Paul Durand"),
        ("LANDLORD_ADDRESS", "3 avenue Foch, 69006 Lyon"),
        ("LANDLORD_EMAIL", "paul.durand@example.fr"),
        ("LANDLORD_CITY", "Lyon"),
        ("SIGNATURE_PATH", signature_path.as_str()),
        ("CHROME_PATH", CHROME_PATH),
        ("SMTP_HOST", "smtp.example.fr"),
        ("SMTP_PORT", "587"),
        ("SMTP_USERNAME", "paul.durand@example.fr"),
        ("SMTP_PASSWORD", "placeholder-for-tests"),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value.to_owned()))
    .collect()
}

pub(crate) fn sample_payload() -> ReceiptInputPayload {
    ReceiptInputPayload {
        tenant_name: "Jeanne Martin".to_owned(),
        tenant_address: "8 rue Victor Hugo, 75011 Paris".to_owned(),
        tenant_email: "jeanne.martin@example.fr".to_owned(),
        property_address: "12 rue des Lilas, 75011 Paris".to_owned(),
        period_start: "2026-10-01".to_owned(),
        period_end: "2026-10-31".to_owned(),
        rent_cents: 65_000,
        charges_cents: 5_050,
        payment_date: "2026-10-05".to_owned(),
    }
}

/// Configuration fournie par une table fixe, ou échec de lecture programmé.
pub(crate) struct FakeConfigSource {
    pub variables: HashMap<String, String>,
    pub failure: Option<fn() -> ConfigError>,
}

impl ConfigSource for FakeConfigSource {
    fn variables(&self) -> Result<HashMap<String, String>, ConfigError> {
        match self.failure {
            Some(failure) => Err(failure()),
            None => Ok(self.variables.clone()),
        }
    }
}

/// Chemin propre à un test dans le dossier temporaire du système, absent au départ.
pub(crate) fn unique_temp_path(test_name: &str) -> PathBuf {
    let nanoseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "quittance-app-{test_name}-{}-{nanoseconds}",
        process::id()
    ))
}

/// Étapes observées par les fakes, dans l'ordre où elles ont eu lieu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DeliveryEvent {
    RendererCreated(PathBuf),
    MailerCreated(SmtpConfig),
    PdfRendered(String),
    PdfArchived { file_name: String, pdf: Vec<u8> },
    EmailSent(ReceiptEmail),
}

pub(crate) type DeliveryJournal = Rc<RefCell<Vec<DeliveryEvent>>>;

/// Fabrique de fakes : chaque étape réussit, sauf celle dont l'échec est programmé.
#[derive(Default)]
pub(crate) struct FakeDelivery {
    pub journal: DeliveryJournal,
    pub pdf: Option<Vec<u8>>,
    pub renderer_creation_failure: Option<fn() -> PdfError>,
    pub rendering_failure: Option<fn() -> PdfError>,
    pub mailer_creation_failure: Option<fn() -> MailError>,
    pub sending_failure: Option<fn() -> MailError>,
}

impl FakeDelivery {
    pub(crate) fn events(&self) -> Vec<DeliveryEvent> {
        self.journal.borrow().clone()
    }
}

impl DeliveryFactory for FakeDelivery {
    fn pdf_renderer(&self, chrome_path: &Path) -> Result<Box<dyn PdfRenderer>, PdfError> {
        if let Some(failure) = self.renderer_creation_failure {
            return Err(failure());
        }
        self.journal
            .borrow_mut()
            .push(DeliveryEvent::RendererCreated(chrome_path.to_owned()));
        Ok(Box::new(FakePdfRenderer {
            journal: Rc::clone(&self.journal),
            pdf: self.pdf.clone().unwrap_or_else(|| FAKE_PDF.to_vec()),
            failure: self.rendering_failure,
        }))
    }

    fn mailer(&self, smtp: &SmtpConfig) -> Result<Box<dyn Mailer>, MailError> {
        if let Some(failure) = self.mailer_creation_failure {
            return Err(failure());
        }
        self.journal
            .borrow_mut()
            .push(DeliveryEvent::MailerCreated(smtp.clone()));
        Ok(Box::new(FakeMailer {
            journal: Rc::clone(&self.journal),
            failure: self.sending_failure,
        }))
    }
}

struct FakePdfRenderer {
    journal: DeliveryJournal,
    pdf: Vec<u8>,
    failure: Option<fn() -> PdfError>,
}

impl PdfRenderer for FakePdfRenderer {
    fn render(&self, html: &str) -> Result<Vec<u8>, PdfError> {
        if let Some(failure) = self.failure {
            return Err(failure());
        }
        self.journal
            .borrow_mut()
            .push(DeliveryEvent::PdfRendered(html.to_owned()));
        Ok(self.pdf.clone())
    }
}

struct FakeMailer {
    journal: DeliveryJournal,
    failure: Option<fn() -> MailError>,
}

impl Mailer for FakeMailer {
    fn send(&self, email: &ReceiptEmail) -> Result<(), MailError> {
        if let Some(failure) = self.failure {
            return Err(failure());
        }
        self.journal
            .borrow_mut()
            .push(DeliveryEvent::EmailSent(email.clone()));
        Ok(())
    }
}

/// Archive qui consigne les PDF dans le journal partagé avec [`FakeDelivery`].
pub(crate) struct FakePdfArchive {
    pub journal: DeliveryJournal,
    pub failure: Option<fn() -> io::Error>,
}

impl SentPdfArchive for FakePdfArchive {
    fn store(&self, file_name: &str, pdf: &[u8]) -> io::Result<()> {
        if let Some(failure) = self.failure {
            return Err(failure());
        }
        self.journal.borrow_mut().push(DeliveryEvent::PdfArchived {
            file_name: file_name.to_owned(),
            pdf: pdf.to_vec(),
        });
        Ok(())
    }
}
