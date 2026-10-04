use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::app_config::{ConfigError, EnvFileProblem};

/// Nom du fichier de configuration, placé à côté de l'exécutable.
const ENV_FILE_NAME: &str = ".env";

/// Provenance des variables de configuration, derrière un trait pour tester les commandes
/// sans toucher à l'environnement réel du processus.
pub trait ConfigSource {
    fn variables(&self) -> Result<HashMap<String, String>, ConfigError>;
}

/// Variables d'un fichier `.env`, complétées par l'environnement du processus, qui prime :
/// une variable exportée au lancement remplace celle du fichier.
pub struct EnvFileConfigSource {
    env_file: PathBuf,
}

impl EnvFileConfigSource {
    pub fn new(env_file: impl Into<PathBuf>) -> Self {
        Self {
            env_file: env_file.into(),
        }
    }

    /// Fichier `.env` du dossier de l'exécutable.
    pub fn next_to_executable() -> Result<Self, ConfigError> {
        let executable = std::env::current_exe().map_err(ConfigError::UnknownExecutableLocation)?;
        let directory = executable.parent().ok_or_else(|| {
            ConfigError::UnknownExecutableLocation(io::Error::other(format!(
                "executable path `{}` has no parent directory",
                executable.display()
            )))
        })?;
        Ok(Self::new(directory.join(ENV_FILE_NAME)))
    }

    pub fn env_file(&self) -> &Path {
        &self.env_file
    }
}

impl ConfigSource for EnvFileConfigSource {
    /// Un fichier absent n'est pas une erreur : seules manqueront ses variables. Une
    /// variable dont le nom ou la valeur n'est pas en UTF-8 est ignorée : elle ne peut pas
    /// appartenir à la configuration.
    fn variables(&self) -> Result<HashMap<String, String>, ConfigError> {
        let mut variables = self.env_file_variables()?;
        variables.extend(std::env::vars_os().filter_map(|(name, value)| {
            Some((name.into_string().ok()?, value.into_string().ok()?))
        }));
        Ok(variables)
    }
}

impl EnvFileConfigSource {
    fn env_file_variables(&self) -> Result<HashMap<String, String>, ConfigError> {
        let entries = match dotenvy::from_path_iter(self.env_file()) {
            Ok(entries) => entries,
            Err(error) if error.not_found() => return Ok(HashMap::new()),
            Err(error) => return Err(self.unreadable(error, 0)),
        };
        let mut variables = HashMap::new();
        for (valid_entries_before, entry) in entries.enumerate() {
            let (name, value) =
                entry.map_err(|error| self.unreadable(error, valid_entries_before))?;
            variables.insert(name, value);
        }
        Ok(variables)
    }

    /// Seule une erreur d'entrée-sortie est conservée : les autres erreurs de dotenvy
    /// recopient la ligne fautive, voire la suite du fichier.
    fn unreadable(&self, error: dotenvy::Error, valid_entries_before: usize) -> ConfigError {
        let problem = match error {
            dotenvy::Error::Io(source) => EnvFileProblem::Io(source),
            _ => EnvFileProblem::MalformedEntry {
                valid_entries_before,
            },
        };
        ConfigError::UnreadableEnvFile {
            path: self.env_file().to_owned(),
            problem,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::test_support::unique_temp_path;

    /// Variable propre à ces tests, absente de l'environnement du processus.
    const FILE_ONLY_VARIABLE: &str = "QUITTANCE_APP_TEST_FILE_ONLY_VARIABLE";

    fn env_file_with(test_name: &str, content: &str) -> PathBuf {
        let path = unique_temp_path(test_name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn given_env_file_when_reading_variables_then_its_variables_are_present() {
        let env_file = env_file_with("env-file", &format!("{FILE_ONLY_VARIABLE}=\"Lyon\"\n"));

        let variables = EnvFileConfigSource::new(&env_file).variables().unwrap();

        assert_eq!(
            variables.get(FILE_ONLY_VARIABLE).map(String::as_str),
            Some("Lyon")
        );
        fs::remove_file(env_file).unwrap();
    }

    #[test]
    fn given_env_file_when_reading_variables_then_process_variables_are_present_too() {
        let env_file = env_file_with("env-file-and-process", &format!("{FILE_ONLY_VARIABLE}=1\n"));

        let variables = EnvFileConfigSource::new(&env_file).variables().unwrap();

        assert_eq!(
            variables.get("CARGO_PKG_NAME").map(String::as_str),
            Some(env!("CARGO_PKG_NAME"))
        );
        fs::remove_file(env_file).unwrap();
    }

    #[test]
    fn given_variable_in_both_file_and_process_when_reading_variables_then_process_value_wins() {
        let env_file = env_file_with("env-file-overridden", "CARGO_PKG_NAME=from-env-file\n");

        let variables = EnvFileConfigSource::new(&env_file).variables().unwrap();

        assert_eq!(
            variables.get("CARGO_PKG_NAME").map(String::as_str),
            Some(env!("CARGO_PKG_NAME"))
        );
        fs::remove_file(env_file).unwrap();
    }

    #[test]
    fn given_missing_env_file_when_reading_variables_then_process_variables_are_returned() {
        let missing_env_file = unique_temp_path("missing-env-file");

        let variables = EnvFileConfigSource::new(missing_env_file)
            .variables()
            .unwrap();

        assert_eq!(
            variables.get("CARGO_PKG_NAME").map(String::as_str),
            Some(env!("CARGO_PKG_NAME"))
        );
        assert!(!variables.contains_key(FILE_ONLY_VARIABLE));
    }

    #[test]
    fn given_single_quoted_value_when_reading_variables_then_it_is_taken_literally() {
        let literal_value = r##"p@$HOME ${USER} \n "#x"##;
        let env_file = env_file_with(
            "single-quoted-env-file",
            &format!("{FILE_ONLY_VARIABLE}='{literal_value}'\n"),
        );

        let variables = EnvFileConfigSource::new(&env_file).variables().unwrap();

        assert_eq!(
            variables.get(FILE_ONLY_VARIABLE).map(String::as_str),
            Some(literal_value)
        );
        fs::remove_file(env_file).unwrap();
    }

    #[test]
    fn given_single_quoted_value_with_escaped_apostrophe_when_reading_variables_then_it_is_kept() {
        let env_file = env_file_with(
            "apostrophe-env-file",
            &format!(r"{FILE_ONLY_VARIABLE}='it'\''s'{}", '\n'),
        );

        let variables = EnvFileConfigSource::new(&env_file).variables().unwrap();

        assert_eq!(
            variables.get(FILE_ONLY_VARIABLE).map(String::as_str),
            Some("it's")
        );
        fs::remove_file(env_file).unwrap();
    }

    #[test]
    fn given_malformed_env_file_when_reading_variables_then_env_file_is_unreadable() {
        let env_file = env_file_with("malformed-env-file", "LANDLORD_CITY='Lyon\n");

        let result = EnvFileConfigSource::new(&env_file).variables();

        assert!(
            matches!(&result, Err(ConfigError::UnreadableEnvFile { path, .. }) if path == &env_file),
            "got {result:?}"
        );
        fs::remove_file(env_file).unwrap();
    }

    #[test]
    fn given_env_file_path_that_is_a_directory_when_reading_variables_then_env_file_is_unreadable()
    {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR"));

        let result = EnvFileConfigSource::new(directory).variables();

        assert!(matches!(result, Err(ConfigError::UnreadableEnvFile { .. })));
    }

    #[test]
    fn given_running_executable_when_locating_env_file_then_it_sits_next_to_the_executable() {
        let executable = std::env::current_exe().unwrap();

        let source = EnvFileConfigSource::next_to_executable().unwrap();

        assert_eq!(source.env_file(), executable.parent().unwrap().join(".env"));
    }
}
