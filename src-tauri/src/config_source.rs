use std::collections::HashMap;

/// Provenance des variables de configuration, derrière un trait pour tester les commandes
/// sans toucher à l'environnement réel du processus.
pub trait ConfigSource {
    fn variables(&self) -> HashMap<String, String>;
}

/// Variables d'environnement du processus.
pub struct EnvironmentConfigSource;

impl ConfigSource for EnvironmentConfigSource {
    /// Une variable dont le nom ou la valeur n'est pas en UTF-8 est ignorée : elle ne peut
    /// pas appartenir à la configuration.
    fn variables(&self) -> HashMap<String, String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_process_environment_when_reading_variables_then_variables_set_by_cargo_are_present() {
        let variables = EnvironmentConfigSource.variables();

        assert_eq!(
            variables.get("CARGO_PKG_NAME").map(String::as_str),
            Some(env!("CARGO_PKG_NAME"))
        );
    }
}
