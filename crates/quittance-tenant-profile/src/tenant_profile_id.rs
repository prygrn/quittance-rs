use std::fmt;

use crate::{TenantProfileError, TenantProfileStorageError};

/// Identifiant d'une fiche, nom de son fichier sans l'extension : minuscules, lettres et
/// chiffres ASCII, mots séparés par un tiret unique.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TenantProfileId(String);

impl TenantProfileId {
    /// Id dérivé du nom du locataire : minuscules, accents français retirés, toute suite
    /// d'autres caractères remplacée par un tiret, sans tiret en tête ni en fin.
    ///
    /// Seules les lettres accentuées précomposées (NFC) sont reconnues : un nom en Unicode
    /// décomposé (NFD), où `é` s'écrit `e` suivi de l'accent combinant U+0301, donne
    /// `helene` pour « Hélène » en NFC mais `he-le-ne` en NFD, l'accent combinant étant un
    /// séparateur. Les saisies clavier et les fichiers écrits par l'app sont en NFC ; la
    /// normalisation demanderait une dépendance.
    pub fn from_tenant_name(tenant_name: &str) -> Result<Self, TenantProfileError> {
        let spelled: String = tenant_name
            .to_lowercase()
            .chars()
            .map(without_french_accent)
            .collect();
        let id = spelled
            .split(|character: char| !character.is_ascii_alphanumeric())
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>()
            .join(ID_WORD_SEPARATOR);
        if id.is_empty() {
            return Err(TenantProfileError::EmptyId {
                tenant_name: tenant_name.to_owned(),
            });
        }
        Ok(Self(id))
    }

    /// Id reçu de l'extérieur (UI, nom de fichier). Seule la forme produite par
    /// [`Self::from_tenant_name`] est acceptée : un id ne désigne jamais un autre chemin.
    pub fn parse(id: &str) -> Result<Self, TenantProfileStorageError> {
        match Self::from_tenant_name(id) {
            Ok(derived) if derived.as_str() == id => Ok(derived),
            _ => Err(TenantProfileStorageError::InvalidId(id.to_owned())),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

const ID_WORD_SEPARATOR: &str = "-";

/// Lettres minuscules accentuées et ligatures du français, avec leur graphie ASCII.
const FRENCH_LETTERS: [(char, &str); 18] = [
    ('à', "a"),
    ('â', "a"),
    ('ä', "a"),
    ('æ', "ae"),
    ('ç', "c"),
    ('é', "e"),
    ('è', "e"),
    ('ê', "e"),
    ('ë', "e"),
    ('î', "i"),
    ('ï', "i"),
    ('ô', "o"),
    ('ö', "o"),
    ('œ', "oe"),
    ('ù', "u"),
    ('û', "u"),
    ('ü', "u"),
    ('ÿ', "y"),
];

fn without_french_accent(character: char) -> String {
    FRENCH_LETTERS
        .iter()
        .find(|(letter, _)| *letter == character)
        .map_or_else(|| character.to_string(), |(_, ascii)| (*ascii).to_owned())
}

impl fmt::Display for TenantProfileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn derived(tenant_name: &str) -> String {
        TenantProfileId::from_tenant_name(tenant_name)
            .unwrap()
            .as_str()
            .to_owned()
    }

    #[test]
    fn given_first_and_last_name_when_deriving_id_then_words_are_lowercase_and_hyphenated() {
        assert_eq!(derived("Jeanne Martin"), "jeanne-martin");
    }

    #[test]
    fn given_french_accented_letters_when_deriving_id_then_accents_are_removed() {
        assert_eq!(derived("àâäéèêëîïôöùûüÿç"), "aaaeeeeiioouuuyc");
        assert_eq!(derived("ÀÂÄÉÈÊËÎÏÔÖÙÛÜŸÇ"), "aaaeeeeiioouuuyc");
    }

    #[test]
    fn given_french_ligatures_when_deriving_id_then_they_are_spelled_out() {
        assert_eq!(derived("Œdipe Lætitia"), "oedipe-laetitia");
        assert_eq!(derived("cœur ÆSOPE"), "coeur-aesope");
    }

    #[test]
    fn given_runs_of_other_characters_when_deriving_id_then_each_run_becomes_one_hyphen() {
        assert_eq!(
            derived("  M. & Mme Dupont--Durand (bât. 3B) !"),
            "m-mme-dupont-durand-bat-3b"
        );
    }

    #[test]
    fn given_non_french_letter_when_deriving_id_then_it_is_a_separator() {
        assert_eq!(derived("Muñoz"), "mu-oz");
    }

    #[test]
    fn given_name_without_ascii_letter_or_digit_when_deriving_id_then_id_is_rejected() {
        for tenant_name in ["", "   ", "!!! ---", "ñ"] {
            let result = TenantProfileId::from_tenant_name(tenant_name);

            assert!(
                matches!(&result, Err(TenantProfileError::EmptyId { tenant_name: name }) if name == tenant_name),
                "{tenant_name:?} got {result:?}"
            );
        }
    }

    #[test]
    fn given_derived_id_when_parsing_then_it_is_accepted() {
        let id = TenantProfileId::parse("jeanne-martin-3b").unwrap();

        assert_eq!(id.as_str(), "jeanne-martin-3b");
    }

    #[test]
    fn given_string_that_derivation_would_change_when_parsing_then_it_is_rejected() {
        for id in [
            "",
            "Jeanne",
            "jeanne martin",
            "jeanne--martin",
            "-jeanne",
            "jeanne-",
            "hélène",
            "../secret",
            "fiches/jeanne",
            "jeanne.toml",
        ] {
            let result = TenantProfileId::parse(id);

            assert!(
                matches!(&result, Err(TenantProfileStorageError::InvalidId(rejected)) if rejected == id),
                "{id:?} got {result:?}"
            );
        }
    }
}
