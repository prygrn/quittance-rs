use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use quittance_core::{Party, PartyError};
use quittance_mailer::{MailError, SmtpConfig};

/// Noms des variables de configuration propres à l'application ; celles du serveur SMTP
/// sont lues par `quittance-mailer`.
pub(crate) mod variable_names {
    pub const LANDLORD_NAME: &str = "LANDLORD_NAME";
    pub const LANDLORD_ADDRESS: &str = "LANDLORD_ADDRESS";
    pub const LANDLORD_EMAIL: &str = "LANDLORD_EMAIL";
    pub const LANDLORD_CITY: &str = "LANDLORD_CITY";
    pub const SIGNATURE_PATH: &str = "SIGNATURE_PATH";
    pub const CHROME_PATH: &str = "CHROME_PATH";
}

/// Configuration absente, illisible ou invalide.
#[derive(Debug)]
pub enum ConfigError {
    MissingVariable(&'static str),
    InvalidLandlord(PartyError),
    InvalidSmtp(MailError),
    /// Le dossier de l'exécutable, où chercher le fichier `.env`, est introuvable.
    UnknownExecutableLocation(io::Error),
    /// Fichier `.env` présent mais illisible ou mal formé.
    UnreadableEnvFile {
        path: PathBuf,
        source: dotenvy::Error,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingVariable(name) => {
                write!(
                    formatter,
                    "environment variable `{name}` is missing or blank"
                )
            }
            Self::InvalidLandlord(source) => {
                write!(formatter, "landlord configuration is invalid: {source}")
            }
            Self::InvalidSmtp(source) => {
                write!(formatter, "SMTP configuration is invalid: {source}")
            }
            Self::UnknownExecutableLocation(source) => write!(
                formatter,
                "executable directory holding the .env file cannot be found: {source}"
            ),
            Self::UnreadableEnvFile { path, source } => write!(
                formatter,
                "env file `{}` cannot be read: {source}",
                path.display()
            ),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingVariable(_) => None,
            Self::InvalidLandlord(source) => Some(source),
            Self::InvalidSmtp(source) => Some(source),
            Self::UnknownExecutableLocation(source) => Some(source),
            Self::UnreadableEnvFile { source, .. } => Some(source),
        }
    }
}

/// Configuration nécessaire pour établir une quittance, donc pour l'aperçu : bailleur,
/// lieu d'émission et signature.
#[derive(Debug, Clone)]
pub struct ReceiptConfig {
    landlord: Party,
    issue_place: String,
    signature_path: PathBuf,
}

impl ReceiptConfig {
    /// Lit la configuration depuis une table de variables, sans accéder elle-même à
    /// l'environnement du processus. Les valeurs sont trimées ; une valeur vide est absente.
    pub fn from_variables(variables: &HashMap<String, String>) -> Result<Self, ConfigError> {
        let landlord = Party::new(
            required_value(variables, variable_names::LANDLORD_NAME)?,
            required_value(variables, variable_names::LANDLORD_ADDRESS)?,
            required_value(variables, variable_names::LANDLORD_EMAIL)?,
        )
        .map_err(ConfigError::InvalidLandlord)?;
        Ok(Self {
            landlord,
            issue_place: required_value(variables, variable_names::LANDLORD_CITY)?.to_owned(),
            signature_path: PathBuf::from(required_value(
                variables,
                variable_names::SIGNATURE_PATH,
            )?),
        })
    }

    /// Bailleur, émetteur de la quittance et destinataire de sa copie cachée.
    pub fn landlord(&self) -> &Party {
        &self.landlord
    }

    /// Lieu de la mention « Fait à …, le … » (`LANDLORD_CITY`).
    pub fn issue_place(&self) -> &str {
        &self.issue_place
    }

    pub fn signature_path(&self) -> &Path {
        &self.signature_path
    }
}

/// Configuration exigée seulement à l'envoi : Chromium pour le PDF, serveur SMTP.
#[derive(Debug, Clone)]
pub struct DeliveryConfig {
    chrome_path: PathBuf,
    smtp: SmtpConfig,
}

impl DeliveryConfig {
    /// Mêmes règles de lecture que [`ReceiptConfig::from_variables`].
    pub fn from_variables(variables: &HashMap<String, String>) -> Result<Self, ConfigError> {
        Ok(Self {
            chrome_path: PathBuf::from(required_value(variables, variable_names::CHROME_PATH)?),
            smtp: SmtpConfig::from_variables(variables).map_err(ConfigError::InvalidSmtp)?,
        })
    }

    pub fn chrome_path(&self) -> &Path {
        &self.chrome_path
    }

    pub fn smtp(&self) -> &SmtpConfig {
        &self.smtp
    }
}

/// Valeur trimée ; une variable vide équivaut à une variable absente.
fn required_value<'map>(
    variables: &'map HashMap<String, String>,
    name: &'static str,
) -> Result<&'map str, ConfigError> {
    variables
        .get(name)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .ok_or(ConfigError::MissingVariable(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{complete_variables, signature_fixture_path};

    const DELIVERY_VARIABLES: [&str; 6] = [
        variable_names::CHROME_PATH,
        "SMTP_HOST",
        "SMTP_PORT",
        "SMTP_USERNAME",
        "SMTP_PASSWORD",
        "SMTP_SECURITY",
    ];

    fn variables_with(name: &str, value: &str) -> HashMap<String, String> {
        let mut variables = complete_variables();
        variables.insert(name.to_owned(), value.to_owned());
        variables
    }

    fn variables_without(name: &str) -> HashMap<String, String> {
        let mut variables = complete_variables();
        variables.remove(name);
        variables
    }

    fn receipt_variables_only() -> HashMap<String, String> {
        let mut variables = complete_variables();
        for name in DELIVERY_VARIABLES {
            variables.remove(name);
        }
        variables
    }

    // Configuration de la quittance

    #[test]
    fn given_complete_variables_when_loading_receipt_config_then_it_carries_every_value() {
        let config = ReceiptConfig::from_variables(&complete_variables()).unwrap();

        assert_eq!(config.landlord().name(), "Paul Durand");
        assert_eq!(config.landlord().address(), "3 avenue Foch, 69006 Lyon");
        assert_eq!(config.landlord().email(), "paul.durand@example.fr");
        assert_eq!(config.issue_place(), "Lyon");
        assert_eq!(config.signature_path(), signature_fixture_path());
    }

    #[test]
    fn given_no_chrome_nor_smtp_variables_when_loading_receipt_config_then_it_is_loaded() {
        let result = ReceiptConfig::from_variables(&receipt_variables_only());

        assert!(result.is_ok(), "got {result:?}");
    }

    #[test]
    fn given_values_surrounded_by_spaces_when_loading_receipt_config_then_they_are_trimmed() {
        let config = ReceiptConfig::from_variables(&variables_with(
            variable_names::LANDLORD_CITY,
            "  Lyon\t",
        ))
        .unwrap();

        assert_eq!(config.issue_place(), "Lyon");
    }

    #[test]
    fn given_each_receipt_variable_absent_when_loading_receipt_config_then_it_is_missing() {
        let required_variables = [
            variable_names::LANDLORD_NAME,
            variable_names::LANDLORD_ADDRESS,
            variable_names::LANDLORD_EMAIL,
            variable_names::LANDLORD_CITY,
            variable_names::SIGNATURE_PATH,
        ];

        for required_variable in required_variables {
            let result = ReceiptConfig::from_variables(&variables_without(required_variable));

            assert!(
                matches!(result, Err(ConfigError::MissingVariable(name)) if name == required_variable),
                "{required_variable} should be missing, got {result:?}"
            );
        }
    }

    #[test]
    fn given_blank_required_variable_when_loading_receipt_config_then_it_is_missing() {
        let result =
            ReceiptConfig::from_variables(&variables_with(variable_names::LANDLORD_CITY, " "));

        assert!(matches!(
            result,
            Err(ConfigError::MissingVariable(variable_names::LANDLORD_CITY))
        ));
    }

    #[test]
    fn given_malformed_landlord_email_when_loading_receipt_config_then_landlord_is_invalid() {
        let result =
            ReceiptConfig::from_variables(&variables_with(variable_names::LANDLORD_EMAIL, "paul"));

        assert!(matches!(
            result,
            Err(ConfigError::InvalidLandlord(PartyError::InvalidEmail(_)))
        ));
    }

    // Configuration de l'envoi

    #[test]
    fn given_complete_variables_when_loading_delivery_config_then_it_carries_every_value() {
        let config = DeliveryConfig::from_variables(&complete_variables()).unwrap();

        assert_eq!(config.chrome_path(), Path::new("/opt/chromium/chrome"));
        assert_eq!(
            config.smtp(),
            &SmtpConfig::from_variables(&complete_variables()).unwrap()
        );
    }

    #[test]
    fn given_chrome_path_surrounded_by_spaces_when_loading_delivery_config_then_it_is_trimmed() {
        let config = DeliveryConfig::from_variables(&variables_with(
            variable_names::CHROME_PATH,
            " /opt/chromium/chrome\n",
        ))
        .unwrap();

        assert_eq!(config.chrome_path(), Path::new("/opt/chromium/chrome"));
    }

    #[test]
    fn given_no_chrome_path_when_loading_delivery_config_then_it_is_missing() {
        let result =
            DeliveryConfig::from_variables(&variables_without(variable_names::CHROME_PATH));

        assert!(matches!(
            result,
            Err(ConfigError::MissingVariable(variable_names::CHROME_PATH))
        ));
    }

    #[test]
    fn given_only_delivery_variables_when_loading_delivery_config_then_it_is_loaded() {
        let variables: HashMap<String, String> = complete_variables()
            .into_iter()
            .filter(|(name, _)| DELIVERY_VARIABLES.contains(&name.as_str()))
            .collect();

        let result = DeliveryConfig::from_variables(&variables);

        assert!(result.is_ok(), "got {result:?}");
    }

    #[test]
    fn given_invalid_smtp_variables_when_loading_delivery_config_then_smtp_is_invalid() {
        let result = DeliveryConfig::from_variables(&variables_with("SMTP_PORT", "smtp"));

        assert!(matches!(
            result,
            Err(ConfigError::InvalidSmtp(MailError::InvalidPort(_)))
        ));
    }

    #[test]
    fn given_missing_smtp_variable_when_loading_delivery_config_then_smtp_is_invalid() {
        let result = DeliveryConfig::from_variables(&variables_without("SMTP_HOST"));

        assert!(matches!(
            result,
            Err(ConfigError::InvalidSmtp(MailError::MissingVariable(
                "SMTP_HOST"
            )))
        ));
    }
}
