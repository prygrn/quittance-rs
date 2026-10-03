use std::collections::HashMap;
use std::fmt;
use std::num::NonZeroU16;

use crate::MailError;

/// Noms des variables de configuration SMTP.
pub(crate) mod variable_names {
    pub const HOST: &str = "SMTP_HOST";
    pub const PORT: &str = "SMTP_PORT";
    pub const USERNAME: &str = "SMTP_USERNAME";
    pub const PASSWORD: &str = "SMTP_PASSWORD";
    pub const SECURITY: &str = "SMTP_SECURITY";
}

/// Chiffrement de la connexion SMTP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SmtpSecurity {
    /// `starttls` : connexion en clair puis passage en TLS, mode par défaut.
    StartTls,
    /// `tls` : TLS dès la connexion.
    ImplicitTls,
    /// `none` : réservé au SMTP local de test (Mailpit), sans authentification.
    Unencrypted,
}

/// Identifiants d'authentification SMTP.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct SmtpCredentials {
    pub(crate) username: String,
    pub(crate) password: String,
}

/// Configuration SMTP validée au chargement.
#[derive(Clone, PartialEq, Eq)]
pub struct SmtpConfig {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) credentials: Option<SmtpCredentials>,
    pub(crate) security: SmtpSecurity,
}

impl SmtpConfig {
    /// Lit la configuration depuis une table de variables (par exemple l'environnement
    /// chargé depuis `.env`), sans accéder elle-même à l'environnement du processus.
    pub fn from_variables(variables: &HashMap<String, String>) -> Result<Self, MailError> {
        let host = required_value(variables, variable_names::HOST)?;
        let port = parse_port(required_value(variables, variable_names::PORT)?)?;
        let username = required_value(variables, variable_names::USERNAME)?;
        let password = required_value(variables, variable_names::PASSWORD)?;
        let security = parse_security(optional_value(variables, variable_names::SECURITY))?;
        Ok(Self {
            host: host.to_owned(),
            port,
            credentials: Some(SmtpCredentials {
                username: username.to_owned(),
                password: password.to_owned(),
            }),
            security,
        })
    }
}

/// Valeur trimée, absente si la variable est vide.
fn optional_value<'map>(variables: &'map HashMap<String, String>, name: &str) -> Option<&'map str> {
    variables
        .get(name)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
}

fn required_value<'map>(
    variables: &'map HashMap<String, String>,
    name: &'static str,
) -> Result<&'map str, MailError> {
    optional_value(variables, name).ok_or(MailError::MissingVariable(name))
}

/// Le port 0 n'est pas joignable : `NonZeroU16` l'écarte avec les valeurs hors bornes.
fn parse_port(value: &str) -> Result<u16, MailError> {
    value
        .parse::<NonZeroU16>()
        .map(NonZeroU16::get)
        .map_err(|_| MailError::InvalidPort(value.to_owned()))
}

fn parse_security(value: Option<&str>) -> Result<SmtpSecurity, MailError> {
    match value {
        None | Some("starttls") => Ok(SmtpSecurity::StartTls),
        Some("tls") => Ok(SmtpSecurity::ImplicitTls),
        Some("none") => Ok(SmtpSecurity::Unencrypted),
        Some(unknown) => Err(MailError::UnknownSecurityMode(unknown.to_owned())),
    }
}

/// Le mot de passe n'apparaît jamais dans les traces.
impl fmt::Debug for SmtpConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SmtpConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field(
                "username",
                &self
                    .credentials
                    .as_ref()
                    .map(|credentials| &credentials.username),
            )
            .field("password", &"<redacted>")
            .field("security", &self.security)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSWORD: &str = "s3cr3t-value";

    fn complete_variables() -> HashMap<String, String> {
        HashMap::from([
            (
                variable_names::HOST.to_owned(),
                "smtp.example.fr".to_owned(),
            ),
            (variable_names::PORT.to_owned(), "587".to_owned()),
            (
                variable_names::USERNAME.to_owned(),
                "paul.durand@example.fr".to_owned(),
            ),
            (variable_names::PASSWORD.to_owned(), PASSWORD.to_owned()),
        ])
    }

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
        let variables = complete_variables();

        let config = SmtpConfig::from_variables(&variables).unwrap();

        assert_eq!(config.host, "smtp.example.fr");
        assert_eq!(config.port, 587);
        let credentials = config.credentials.unwrap();
        assert_eq!(credentials.username, "paul.durand@example.fr");
        assert_eq!(credentials.password, PASSWORD);
    }

    #[test]
    fn given_no_security_variable_when_loading_config_then_starttls_is_used() {
        let variables = complete_variables();

        let config = SmtpConfig::from_variables(&variables).unwrap();

        assert_eq!(config.security, SmtpSecurity::StartTls);
    }

    #[test]
    fn given_blank_security_variable_when_loading_config_then_starttls_is_used() {
        let variables = variables_with(variable_names::SECURITY, "  ");

        let config = SmtpConfig::from_variables(&variables).unwrap();

        assert_eq!(config.security, SmtpSecurity::StartTls);
    }

    #[test]
    fn given_each_known_security_mode_when_loading_config_then_it_is_selected() {
        let security_modes = [
            ("starttls", SmtpSecurity::StartTls),
            ("tls", SmtpSecurity::ImplicitTls),
            ("none", SmtpSecurity::Unencrypted),
        ];

        for (value, expected_security) in security_modes {
            let mut variables = variables_with(variable_names::SECURITY, value);
            variables.insert(variable_names::HOST.to_owned(), "localhost".to_owned());

            let config = SmtpConfig::from_variables(&variables).unwrap();

            assert_eq!(config.security, expected_security, "for `{value}`");
        }
    }

    #[test]
    fn given_values_surrounded_by_spaces_when_loading_config_then_they_are_trimmed() {
        let mut variables = variables_with(variable_names::HOST, " smtp.example.fr\t");
        variables.insert(variable_names::PORT.to_owned(), " 465 ".to_owned());
        variables.insert(variable_names::SECURITY.to_owned(), " tls\n".to_owned());
        variables.insert(
            variable_names::USERNAME.to_owned(),
            " paul.durand@example.fr ".to_owned(),
        );

        let config = SmtpConfig::from_variables(&variables).unwrap();

        assert_eq!(config.host, "smtp.example.fr");
        assert_eq!(config.port, 465);
        assert_eq!(config.security, SmtpSecurity::ImplicitTls);
        assert_eq!(
            config.credentials.unwrap().username,
            "paul.durand@example.fr"
        );
    }

    #[test]
    fn given_password_surrounded_by_spaces_when_loading_config_then_it_is_kept_as_is() {
        let variables = variables_with(variable_names::PASSWORD, " s3cr3t value\t");

        let config = SmtpConfig::from_variables(&variables).unwrap();

        assert_eq!(config.credentials.unwrap().password, " s3cr3t value\t");
    }

    #[test]
    fn given_empty_password_when_loading_config_then_it_is_missing() {
        let variables = variables_with(variable_names::PASSWORD, "");

        let result = SmtpConfig::from_variables(&variables);

        assert!(matches!(
            result,
            Err(MailError::MissingVariable(variable_names::PASSWORD))
        ));
    }

    #[test]
    fn given_tls_mode_without_each_credential_when_loading_config_then_it_is_missing() {
        for credential_variable in [variable_names::USERNAME, variable_names::PASSWORD] {
            let mut variables = variables_without(credential_variable);
            variables.insert(variable_names::SECURITY.to_owned(), "tls".to_owned());

            let result = SmtpConfig::from_variables(&variables);

            assert!(
                matches!(result, Err(MailError::MissingVariable(name)) if name == credential_variable),
                "{credential_variable} should be missing, got {result:?}"
            );
        }
    }

    #[test]
    fn given_unencrypted_mode_towards_each_loopback_host_when_loading_config_then_it_is_accepted() {
        for loopback_host in ["localhost", "127.0.0.1", "::1"] {
            let mut variables = variables_with(variable_names::SECURITY, "none");
            variables.insert(variable_names::HOST.to_owned(), loopback_host.to_owned());

            let result = SmtpConfig::from_variables(&variables);

            assert!(
                result.is_ok(),
                "{loopback_host} should be accepted, got {result:?}"
            );
        }
    }

    #[test]
    fn given_unencrypted_mode_towards_remote_hosts_when_loading_config_then_each_is_rejected() {
        for remote_host in [
            "smtp.example.fr",
            "192.168.1.10",
            "127.0.0.2",
            "localhost.example.fr",
        ] {
            let mut variables = variables_with(variable_names::SECURITY, "none");
            variables.insert(variable_names::HOST.to_owned(), remote_host.to_owned());

            let result = SmtpConfig::from_variables(&variables);

            assert!(
                matches!(result, Err(MailError::UnencryptedRemoteHost(_))),
                "{remote_host} should be rejected, got {result:?}"
            );
        }
    }

    #[test]
    fn given_unencrypted_mode_without_credentials_when_loading_config_then_it_is_accepted() {
        let mut variables = variables_without(variable_names::USERNAME);
        variables.remove(variable_names::PASSWORD);
        variables.insert(variable_names::HOST.to_owned(), "localhost".to_owned());
        variables.insert(variable_names::SECURITY.to_owned(), "none".to_owned());

        let config = SmtpConfig::from_variables(&variables).unwrap();

        assert!(config.credentials.is_none());
    }

    #[test]
    fn given_unencrypted_mode_with_credentials_when_loading_config_then_they_are_not_kept() {
        let mut variables = variables_with(variable_names::SECURITY, "none");
        variables.insert(variable_names::HOST.to_owned(), "localhost".to_owned());

        let config = SmtpConfig::from_variables(&variables).unwrap();

        assert!(config.credentials.is_none());
    }

    #[test]
    fn given_each_required_variable_absent_when_loading_config_then_it_is_missing() {
        let required_variables = [
            variable_names::HOST,
            variable_names::PORT,
            variable_names::USERNAME,
            variable_names::PASSWORD,
        ];

        for required_variable in required_variables {
            let variables = variables_without(required_variable);

            let result = SmtpConfig::from_variables(&variables);

            assert!(
                matches!(result, Err(MailError::MissingVariable(name)) if name == required_variable),
                "{required_variable} should be missing, got {result:?}"
            );
        }
    }

    #[test]
    fn given_blank_required_variable_when_loading_config_then_it_is_missing() {
        let variables = variables_with(variable_names::HOST, "   ");

        let result = SmtpConfig::from_variables(&variables);

        assert!(matches!(
            result,
            Err(MailError::MissingVariable(variable_names::HOST))
        ));
    }

    #[test]
    fn given_invalid_ports_when_loading_config_then_each_is_rejected() {
        let invalid_ports = ["smtp", "-1", "0", "65536", "58 7", "587.0"];

        for invalid_port in invalid_ports {
            let variables = variables_with(variable_names::PORT, invalid_port);

            let result = SmtpConfig::from_variables(&variables);

            assert!(
                matches!(result, Err(MailError::InvalidPort(_))),
                "{invalid_port} should be rejected, got {result:?}"
            );
        }
    }

    #[test]
    fn given_boundary_ports_when_loading_config_then_each_is_accepted() {
        for boundary_port in ["1", "65535"] {
            let variables = variables_with(variable_names::PORT, boundary_port);

            let result = SmtpConfig::from_variables(&variables);

            assert!(result.is_ok(), "{boundary_port} should be accepted");
        }
    }

    #[test]
    fn given_unknown_security_modes_when_loading_config_then_each_is_rejected() {
        let unknown_security_modes = ["ssl", "STARTTLS", "plain", "starttls tls"];

        for unknown_security_mode in unknown_security_modes {
            let variables = variables_with(variable_names::SECURITY, unknown_security_mode);

            let result = SmtpConfig::from_variables(&variables);

            assert!(
                matches!(result, Err(MailError::UnknownSecurityMode(_))),
                "{unknown_security_mode} should be rejected, got {result:?}"
            );
        }
    }

    #[test]
    fn given_loaded_config_when_debug_formatting_then_password_is_hidden() {
        let config = SmtpConfig::from_variables(&complete_variables()).unwrap();

        let debug_output = format!("{config:?}");

        assert!(!debug_output.contains(PASSWORD));
        assert!(debug_output.contains("smtp.example.fr"));
    }
}
