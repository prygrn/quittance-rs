use std::error::Error;
use std::fmt;

use quittance_core::ReceiptError;
use quittance_mailer::MailError;
use quittance_pdf::PdfError;
use quittance_signature::SignatureError;
use quittance_template::TemplateError;
use serde::Serialize;

use crate::app_config::ConfigError;
use crate::receipt_input_payload::ReceiptInputError;

/// Source d'échec d'une commande, miroir exact de `CommandErrorCode` dans `src/api.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandErrorCode {
    Validation,
    Template,
    Signature,
    Pdf,
    Mail,
    Config,
    Unknown,
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
        let _ = formatter;
        todo!()
    }
}

impl From<ReceiptInputError> for CommandError {
    fn from(error: ReceiptInputError) -> Self {
        let _ = error;
        todo!()
    }
}

impl From<ReceiptError> for CommandError {
    fn from(error: ReceiptError) -> Self {
        let _ = error;
        todo!()
    }
}

impl From<ConfigError> for CommandError {
    fn from(error: ConfigError) -> Self {
        let _ = error;
        todo!()
    }
}

impl From<SignatureError> for CommandError {
    fn from(error: SignatureError) -> Self {
        let _ = error;
        todo!()
    }
}

impl From<TemplateError> for CommandError {
    fn from(error: TemplateError) -> Self {
        let _ = error;
        todo!()
    }
}

impl From<PdfError> for CommandError {
    fn from(error: PdfError) -> Self {
        Self::from_error(CommandErrorCode::Pdf, &error)
    }
}

impl From<MailError> for CommandError {
    fn from(error: MailError) -> Self {
        let _ = error;
        todo!()
    }
}

/// Message de l'erreur suivi de ses causes, chacune omise si le message la reprend déjà.
fn message_with_causes(error: &dyn Error) -> String {
    let _ = error;
    todo!()
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::PathBuf;

    use quittance_core::PartyError;
    use serde::Deserialize;
    use tauri::ipc::InvokeError;

    use super::*;
    use crate::test_support::{date, from_ipc_json, to_ipc_json};

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
    fn given_receipt_input_error_when_converting_then_code_is_validation() {
        let error = ReceiptInputError::InvalidIsoDate {
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
