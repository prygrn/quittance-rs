use std::collections::HashMap;
use std::error::Error;
use std::fmt;
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

/// Configuration absente ou invalide.
#[derive(Debug)]
pub enum ConfigError {
    MissingVariable(&'static str),
    InvalidLandlord(PartyError),
    InvalidSmtp(MailError),
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
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingVariable(_) => None,
            Self::InvalidLandlord(source) => Some(source),
            Self::InvalidSmtp(source) => Some(source),
        }
    }
}

/// Configuration complète de l'application, validée au chargement.
#[derive(Debug, Clone)]
pub struct AppConfig {
    landlord: Party,
    issue_place: String,
    signature_path: PathBuf,
    chrome_path: PathBuf,
    smtp: SmtpConfig,
}

impl AppConfig {
    /// Lit la configuration depuis une table de variables, sans accéder elle-même à
    /// l'environnement du processus. Les valeurs sont trimées ; une valeur vide est absente.
    pub fn from_variables(variables: &HashMap<String, String>) -> Result<Self, ConfigError> {
        let _ = variables;
        todo!()
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

    pub fn chrome_path(&self) -> &Path {
        &self.chrome_path
    }

    pub fn smtp(&self) -> &SmtpConfig {
        &self.smtp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{complete_variables, signature_fixture_path};

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

    #[test]
    fn given_complete_variables_when_loading_config_then_it_carries_every_value() {
        let config = AppConfig::from_variables(&complete_variables()).unwrap();

        assert_eq!(config.landlord().name(), "Paul Durand");
        assert_eq!(config.landlord().address(), "3 avenue Foch, 69006 Lyon");
        assert_eq!(config.landlord().email(), "paul.durand@example.fr");
        assert_eq!(config.issue_place(), "Lyon");
        assert_eq!(config.signature_path(), signature_fixture_path());
        assert_eq!(config.chrome_path(), Path::new("/opt/chromium/chrome"));
    }

    #[test]
    fn given_complete_variables_when_loading_config_then_smtp_config_is_read_by_the_mailer() {
        let config = AppConfig::from_variables(&complete_variables()).unwrap();

        assert_eq!(
            config.smtp(),
            &SmtpConfig::from_variables(&complete_variables()).unwrap()
        );
    }

    #[test]
    fn given_values_surrounded_by_spaces_when_loading_config_then_they_are_trimmed() {
        let mut variables = variables_with(variable_names::LANDLORD_CITY, "  Lyon\t");
        variables.insert(
            variable_names::CHROME_PATH.to_owned(),
            " /opt/chromium/chrome\n".to_owned(),
        );

        let config = AppConfig::from_variables(&variables).unwrap();

        assert_eq!(config.issue_place(), "Lyon");
        assert_eq!(config.chrome_path(), Path::new("/opt/chromium/chrome"));
    }

    #[test]
    fn given_each_required_variable_absent_when_loading_config_then_it_is_missing() {
        let required_variables = [
            variable_names::LANDLORD_NAME,
            variable_names::LANDLORD_ADDRESS,
            variable_names::LANDLORD_EMAIL,
            variable_names::LANDLORD_CITY,
            variable_names::SIGNATURE_PATH,
            variable_names::CHROME_PATH,
        ];

        for required_variable in required_variables {
            let result = AppConfig::from_variables(&variables_without(required_variable));

            assert!(
                matches!(result, Err(ConfigError::MissingVariable(name)) if name == required_variable),
                "{required_variable} should be missing, got {result:?}"
            );
        }
    }

    #[test]
    fn given_blank_required_variable_when_loading_config_then_it_is_missing() {
        let result = AppConfig::from_variables(&variables_with(variable_names::LANDLORD_CITY, " "));

        assert!(matches!(
            result,
            Err(ConfigError::MissingVariable(variable_names::LANDLORD_CITY))
        ));
    }

    #[test]
    fn given_malformed_landlord_email_when_loading_config_then_landlord_is_invalid() {
        let result =
            AppConfig::from_variables(&variables_with(variable_names::LANDLORD_EMAIL, "paul"));

        assert!(matches!(
            result,
            Err(ConfigError::InvalidLandlord(PartyError::InvalidEmail(_)))
        ));
    }

    #[test]
    fn given_invalid_smtp_variables_when_loading_config_then_smtp_is_invalid() {
        let result = AppConfig::from_variables(&variables_with("SMTP_PORT", "smtp"));

        assert!(matches!(
            result,
            Err(ConfigError::InvalidSmtp(MailError::InvalidPort(_)))
        ));
    }

    #[test]
    fn given_missing_smtp_variable_when_loading_config_then_smtp_is_invalid() {
        let result = AppConfig::from_variables(&variables_without("SMTP_HOST"));

        assert!(matches!(
            result,
            Err(ConfigError::InvalidSmtp(MailError::MissingVariable(
                "SMTP_HOST"
            )))
        ));
    }
}
