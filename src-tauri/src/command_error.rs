use std::error::Error;
use std::fmt;

use quittance_core::ReceiptError;
use quittance_mailer::MailError;
use quittance_pdf::PdfError;
use quittance_signature::SignatureError;
use quittance_template::TemplateError;
use quittance_tenant_profile::{TenantProfileError, TenantProfileStorageError};
use serde::{Serialize, Serializer};

use crate::app_config::ConfigError;
use crate::iso_date::InvalidIsoDate;

/// Source d'échec d'une commande, miroir exact de `CommandErrorCode` dans `src/api.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandErrorCode {
    Validation,
    Template,
    Signature,
    Pdf,
    Mail,
    Config,
    TenantProfile,
    TenantProfileExists,
    Unknown,
}

impl CommandErrorCode {
    /// Valeur reçue par l'UI.
    pub fn api_name(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::Template => "template",
            Self::Signature => "signature",
            Self::Pdf => "pdf",
            Self::Mail => "mail",
            Self::Config => "config",
            Self::TenantProfile => "tenantProfile",
            Self::TenantProfileExists => "tenantProfileExists",
            Self::Unknown => "unknown",
        }
    }
}

impl Serialize for CommandErrorCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.api_name())
    }
}

/// Erreur d'une commande Tauri : l'UI reçoit un rejet `{ code, message }`. Le message,
/// en anglais, sert au débogage ; l'UI choisit son libellé d'après le seul code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandError {
    code: CommandErrorCode,
    message: String,
}

impl CommandError {
    pub fn new(code: CommandErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn code(&self) -> CommandErrorCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    fn from_error(code: CommandErrorCode, error: &dyn Error) -> Self {
        Self::new(code, message_with_causes(error))
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "[{}] {}", self.code().api_name(), self.message())
    }
}

impl From<InvalidIsoDate> for CommandError {
    fn from(error: InvalidIsoDate) -> Self {
        Self::from_error(CommandErrorCode::Validation, &error)
    }
}

impl From<ReceiptError> for CommandError {
    fn from(error: ReceiptError) -> Self {
        Self::from_error(CommandErrorCode::Validation, &error)
    }
}

impl From<ConfigError> for CommandError {
    fn from(error: ConfigError) -> Self {
        Self::from_error(CommandErrorCode::Config, &error)
    }
}

impl From<SignatureError> for CommandError {
    fn from(error: SignatureError) -> Self {
        Self::from_error(CommandErrorCode::Signature, &error)
    }
}

/// Un montant impossible à écrire en lettres relève aussi du rendu : `src/api.ts` réserve
/// `validation` aux refus de `quittance-core`.
impl From<TemplateError> for CommandError {
    fn from(error: TemplateError) -> Self {
        Self::from_error(CommandErrorCode::Template, &error)
    }
}

impl From<PdfError> for CommandError {
    fn from(error: PdfError) -> Self {
        Self::from_error(CommandErrorCode::Pdf, &error)
    }
}

/// Une configuration SMTP invalide arrive enveloppée dans [`ConfigError`] ; toute autre
/// erreur d'email relève de l'envoi.
impl From<MailError> for CommandError {
    fn from(error: MailError) -> Self {
        Self::from_error(CommandErrorCode::Mail, &error)
    }
}

/// Saisie de fiche invalide à l'enregistrement. Au chargement, la même erreur arrive
/// enveloppée dans [`TenantProfileStorageError`].
impl From<TenantProfileError> for CommandError {
    fn from(error: TenantProfileError) -> Self {
        Self::from_error(CommandErrorCode::Validation, &error)
    }
}

impl From<TenantProfileStorageError> for CommandError {
    fn from(error: TenantProfileStorageError) -> Self {
        let code = match error {
            TenantProfileStorageError::AlreadyExists { .. } => {
                CommandErrorCode::TenantProfileExists
            }
            _ => CommandErrorCode::TenantProfile,
        };
        Self::from_error(code, &error)
    }
}

/// Message de l'erreur suivi de ses causes, chacune omise si le message la reprend déjà.
fn message_with_causes(error: &dyn Error) -> String {
    let mut message = error.to_string();
    let mut cause = error.source();
    while let Some(source) = cause {
        let source_message = source.to_string();
        if !message.contains(&source_message) {
            message.push_str(": ");
            message.push_str(&source_message);
        }
        cause = source.source();
    }
    message
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::PathBuf;

    use quittance_core::{Party, PartyError, validate_receipt};
    use quittance_template::{IssueDetails, render_html};
    use serde::Deserialize;
    use tauri::ipc::InvokeError;

    use super::*;
    use crate::app_config::EnvFileProblem;
    use quittance_tenant_profile::TenantProfileRepository;
    use quittance_tenant_profile::test_support::FakeTenantProfileStorage;

    use crate::test_support::{date, from_ipc_json, sample_payload, to_ipc_json};

    /// Erreur produite par `quittance-template` pour un total trop grand pour être écrit
    /// en lettres.
    fn amount_in_words_error() -> TemplateError {
        let landlord = Party::new(
            "Paul Durand",
            "3 avenue Foch, 69006 Lyon",
            "paul.durand@example.fr",
        )
        .unwrap();
        let mut input = sample_payload().into_receipt_input().unwrap();
        input.rent_cents = u64::MAX - input.charges_cents;
        let receipt = validate_receipt(landlord, input).unwrap();
        let issue = IssueDetails {
            place: "Lyon".to_owned(),
            date: date(2026, 10, 6),
        };
        let error = render_html("standard", &receipt, &issue, None).unwrap_err();
        assert!(
            matches!(error, TemplateError::AmountInWords(_)),
            "got {error:?}"
        );
        error
    }

    /// Forme du rejet vue par l'UI.
    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RejectedCommand {
        code: String,
        message: String,
    }

    /// Sérialise l'erreur comme Tauri le fait pour une commande en échec.
    fn rejected(error: CommandError) -> RejectedCommand {
        from_ipc_json(&to_ipc_json(InvokeError::from(error).0)).unwrap()
    }

    #[test]
    fn given_each_code_when_rejecting_command_then_ui_receives_its_api_name() {
        let codes = [
            (CommandErrorCode::Validation, "validation"),
            (CommandErrorCode::Template, "template"),
            (CommandErrorCode::Signature, "signature"),
            (CommandErrorCode::Pdf, "pdf"),
            (CommandErrorCode::Mail, "mail"),
            (CommandErrorCode::Config, "config"),
            (CommandErrorCode::TenantProfile, "tenantProfile"),
            (CommandErrorCode::TenantProfileExists, "tenantProfileExists"),
            (CommandErrorCode::Unknown, "unknown"),
        ];

        for (code, api_name) in codes {
            let rejection = rejected(CommandError::new(code, "debug details"));

            assert_eq!(rejection.code, api_name);
        }
    }

    #[test]
    fn given_command_error_when_rejecting_command_then_ui_receives_its_message() {
        let rejection = rejected(CommandError::new(
            CommandErrorCode::Mail,
            "connection refused",
        ));

        assert_eq!(rejection.message, "connection refused");
    }

    #[test]
    fn given_invalid_iso_date_when_converting_then_code_is_validation() {
        let error = InvalidIsoDate {
            field: "periodStart",
            value: "01/10/2026".to_owned(),
        };

        let command_error = CommandError::from(error);

        assert_eq!(command_error.code(), CommandErrorCode::Validation);
        assert!(command_error.message().contains("periodStart"));
        assert!(command_error.message().contains("01/10/2026"));
    }

    #[test]
    fn given_receipt_errors_when_converting_then_code_is_validation() {
        let errors = [
            ReceiptError::MissingPropertyAddress,
            ReceiptError::InvalidTenant(PartyError::MissingEmail),
            ReceiptError::InvertedPeriod {
                start: date(2026, 10, 31),
                end: date(2026, 10, 1),
            },
            ReceiptError::TotalOverflow {
                rent_cents: u64::MAX,
                charges_cents: 1,
            },
        ];

        for error in errors {
            let command_error = CommandError::from(error);

            assert_eq!(command_error.code(), CommandErrorCode::Validation);
        }
    }

    #[test]
    fn given_config_errors_when_converting_then_code_is_config() {
        let errors = [
            ConfigError::MissingVariable("LANDLORD_CITY"),
            ConfigError::InvalidLandlord(PartyError::InvalidEmail("paul".to_owned())),
            ConfigError::InvalidSmtp(MailError::InvalidPort("smtp".to_owned())),
            ConfigError::UnknownExecutableLocation(io::Error::from(io::ErrorKind::NotFound)),
            ConfigError::UnreadableEnvFile {
                path: PathBuf::from("/opt/quittance/.env"),
                problem: EnvFileProblem::MalformedEntry {
                    valid_entries_before: 3,
                },
            },
        ];

        for error in errors {
            let command_error = CommandError::from(error);

            assert_eq!(command_error.code(), CommandErrorCode::Config);
        }
    }

    #[test]
    fn given_missing_variable_when_converting_then_message_names_it() {
        let command_error = CommandError::from(ConfigError::MissingVariable("LANDLORD_CITY"));

        assert!(command_error.message().contains("LANDLORD_CITY"));
    }

    #[test]
    fn given_unreadable_env_file_when_converting_then_message_names_the_file() {
        let command_error = CommandError::from(ConfigError::UnreadableEnvFile {
            path: PathBuf::from("/opt/quittance/.env"),
            problem: EnvFileProblem::MalformedEntry {
                valid_entries_before: 3,
            },
        });

        assert!(command_error.message().contains("/opt/quittance/.env"));
    }

    #[test]
    fn given_signature_errors_when_converting_then_code_is_signature() {
        let path = PathBuf::from("/nonexistent/signature.png");
        let errors = [
            SignatureError::Unreadable {
                path: path.clone(),
                source: io::Error::from(io::ErrorKind::NotFound),
            },
            SignatureError::TooLarge { path: path.clone() },
            SignatureError::ImageTooLarge { path: path.clone() },
            SignatureError::UnsupportedFormat { path },
        ];

        for error in errors {
            let command_error = CommandError::from(error);

            assert_eq!(command_error.code(), CommandErrorCode::Signature);
        }
    }

    #[test]
    fn given_template_errors_when_converting_then_code_is_template() {
        let errors = [
            TemplateError::UnknownTemplate("fancy".to_owned()),
            TemplateError::InvalidSignature,
            TemplateError::Rendering("broken template".into()),
            amount_in_words_error(),
        ];

        for error in errors {
            let command_error = CommandError::from(error);

            assert_eq!(command_error.code(), CommandErrorCode::Template);
        }
    }

    #[test]
    fn given_pdf_errors_when_converting_then_code_is_pdf() {
        let errors = [
            PdfError::ChromeNotFound {
                path: PathBuf::from("/nonexistent/chrome"),
            },
            PdfError::HtmlTooLarge { size: 2, max: 1 },
            PdfError::BrowserLaunch {
                path: PathBuf::from("/opt/chromium/chrome"),
                source: "chrome exited".into(),
            },
            PdfError::TabCreation("no tab".into()),
            PdfError::Rendering("chrome crashed".into()),
        ];

        for error in errors {
            let command_error = CommandError::from(error);

            assert_eq!(command_error.code(), CommandErrorCode::Pdf);
        }
    }

    #[test]
    fn given_mail_errors_when_converting_then_code_is_mail() {
        let errors = [
            MailError::EmptyAttachment,
            MailError::TransportSetup("no transport".into()),
            MailError::InvalidMessage("bad header".into()),
            MailError::Delivery("connection refused".into()),
        ];

        for error in errors {
            let command_error = CommandError::from(error);

            assert_eq!(command_error.code(), CommandErrorCode::Mail);
        }
    }

    #[test]
    fn given_tenant_profile_errors_when_converting_then_code_is_validation() {
        let errors = [
            TenantProfileError::UnknownTemplate("fancy".to_owned()),
            TenantProfileError::InvalidTenant(PartyError::MissingEmail),
            TenantProfileError::MissingPropertyAddress,
            TenantProfileError::TotalOverflow {
                rent_cents: u64::MAX,
                charges_cents: 1,
            },
            TenantProfileError::EmptyId {
                tenant_name: "!!!".to_owned(),
            },
            TenantProfileError::InvalidAmount {
                field: "rent",
                value: "650.00".to_owned(),
            },
        ];

        for error in errors {
            let command_error = CommandError::from(error);

            assert_eq!(command_error.code(), CommandErrorCode::Validation);
        }
    }

    #[test]
    fn given_tenant_profile_storage_errors_when_converting_then_code_is_tenant_profile() {
        let id = || "jeanne-martin".to_owned();
        let errors = [
            (
                Some("../.env"),
                TenantProfileStorageError::InvalidId("../.env".to_owned()),
            ),
            (
                Some("jeanne-martin"),
                TenantProfileStorageError::NotFound { id: id() },
            ),
            (
                None,
                TenantProfileStorageError::Unlistable(io::Error::from(
                    io::ErrorKind::PermissionDenied,
                )),
            ),
            (
                Some("jeanne-martin"),
                TenantProfileStorageError::Unreadable {
                    id: id(),
                    source: io::Error::from(io::ErrorKind::InvalidData),
                },
            ),
            (
                Some("jeanne-martin"),
                TenantProfileStorageError::Invalid {
                    id: id(),
                    source: TenantProfileError::MissingPropertyAddress,
                },
            ),
            (
                Some("jeanne-martin"),
                TenantProfileStorageError::Unwritable {
                    id: id(),
                    source: io::Error::from(io::ErrorKind::PermissionDenied),
                },
            ),
        ];

        for (expected_id, error) in errors {
            let command_error = CommandError::from(error);

            assert_eq!(command_error.code(), CommandErrorCode::TenantProfile);
            if let Some(expected_id) = expected_id {
                assert!(
                    command_error.message().contains(expected_id),
                    "{command_error} should name {expected_id}"
                );
            }
        }
    }

    #[test]
    fn given_malformed_tenant_profile_file_when_converting_then_code_is_tenant_profile() {
        let storage =
            FakeTenantProfileStorage::with_files([("jeanne-martin", "rent = ".to_owned())]);
        let repository = TenantProfileRepository {
            storage: &storage,
            known_template_ids: &[],
        };
        let error = repository.load("jeanne-martin").unwrap_err();
        assert!(
            matches!(error, TenantProfileStorageError::Malformed { .. }),
            "got {error:?}"
        );

        let command_error = CommandError::from(error);

        assert_eq!(command_error.code(), CommandErrorCode::TenantProfile);
    }

    #[test]
    fn given_existing_tenant_profile_when_converting_then_code_is_tenant_profile_exists() {
        let error = TenantProfileStorageError::AlreadyExists {
            id: "jeanne-martin".to_owned(),
        };

        let command_error = CommandError::from(error);

        assert_eq!(command_error.code(), CommandErrorCode::TenantProfileExists);
        assert!(command_error.message().contains("jeanne-martin"));
    }

    #[test]
    fn given_error_with_silent_cause_when_converting_then_message_includes_the_cause() {
        let command_error = CommandError::from(PdfError::Rendering("chrome crashed".into()));

        assert!(command_error.message().contains("chrome crashed"));
    }

    #[test]
    fn given_error_quoting_its_cause_when_converting_then_cause_appears_once() {
        let command_error = CommandError::from(MailError::Delivery("connection refused".into()));

        assert_eq!(
            command_error
                .message()
                .matches("connection refused")
                .count(),
            1
        );
    }

    #[test]
    fn given_command_error_when_displaying_then_code_and_message_appear() {
        let command_error = CommandError::new(CommandErrorCode::Pdf, "chrome crashed");

        let displayed = command_error.to_string();

        assert!(displayed.contains("pdf"));
        assert!(displayed.contains("chrome crashed"));
    }
}
