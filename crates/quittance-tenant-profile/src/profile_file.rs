//! Format TOML d'un fichier de fiche, montants en euros à la française.

use crate::french_amount::{formatted_amount, parsed_amount};
use crate::{TenantProfile, TenantProfileId, TenantProfileStorageError};

/// Contenu du fichier de la fiche.
pub(crate) fn profile_toml(profile: &TenantProfile) -> Result<String, toml::ser::Error> {
    let _ = (profile, formatted_amount);
    todo!()
}

/// Fiche lue depuis le contenu de son fichier, validée comme à l'enregistrement.
pub(crate) fn profile_from_toml(
    id: &TenantProfileId,
    contents: &str,
    known_template_ids: &[&str],
) -> Result<TenantProfile, TenantProfileStorageError> {
    let _ = (id, contents, known_template_ids, parsed_amount);
    todo!()
}

#[cfg(test)]
mod tests {
    use quittance_core::PartyError;

    use super::*;
    use crate::TenantProfileError;
    use crate::test_support::{KNOWN_TEMPLATE_IDS, SAMPLE_TOML, sample_profile};

    fn jeanne() -> TenantProfileId {
        TenantProfileId::parse("jeanne-martin").unwrap()
    }

    fn loaded(contents: &str) -> Result<TenantProfile, TenantProfileStorageError> {
        profile_from_toml(&jeanne(), contents, KNOWN_TEMPLATE_IDS)
    }

    #[test]
    fn given_profile_when_writing_file_then_each_field_is_a_key_with_french_amounts() {
        let contents = profile_toml(&sample_profile()).unwrap();

        assert_eq!(contents, SAMPLE_TOML);
    }

    #[test]
    fn given_written_file_when_reading_it_back_then_same_profile_is_returned() {
        let contents = profile_toml(&sample_profile()).unwrap();

        assert_eq!(loaded(&contents).unwrap(), sample_profile());
    }

    #[test]
    fn given_short_french_amounts_when_reading_file_then_cents_are_read() {
        let contents = SAMPLE_TOML
            .replace("\"650,00\"", "\"650\"")
            .replace("\"50,50\"", "\"50,5\"");

        let profile = loaded(&contents).unwrap();

        assert_eq!(profile.rent().cents(), 65_000);
        assert_eq!(profile.charges().cents(), 5_050);
    }

    #[test]
    fn given_file_that_is_not_a_profile_when_reading_then_it_is_malformed() {
        let malformed_contents = [
            ("invalid TOML", "tenant_name = \"Jeanne".to_owned()),
            (
                "unknown key",
                format!("{SAMPLE_TOML}deposit = \"1300,00\"\n"),
            ),
            (
                "missing key",
                SAMPLE_TOML.replace("charges = \"50,50\"\n", ""),
            ),
            (
                "number instead of text",
                SAMPLE_TOML.replace("\"650,00\"", "650"),
            ),
        ];

        for (case, contents) in malformed_contents {
            let result = loaded(&contents);

            assert!(
                matches!(&result, Err(TenantProfileStorageError::Malformed { id, .. }) if id == "jeanne-martin"),
                "{case}: got {result:?}"
            );
        }
    }

    #[test]
    fn given_invalid_amount_in_file_when_reading_then_value_is_invalid() {
        let contents = SAMPLE_TOML.replace("\"650,00\"", "\"650.00\"");

        let result = loaded(&contents);

        assert!(
            matches!(
                &result,
                Err(TenantProfileStorageError::Invalid {
                    source: TenantProfileError::InvalidAmount { field: "rent", value },
                    ..
                }) if value == "650.00"
            ),
            "got {result:?}"
        );
    }

    #[test]
    fn given_invalid_tenant_in_file_when_reading_then_value_is_invalid() {
        let contents = SAMPLE_TOML.replace("jeanne.martin@example.fr", "jeanne.martin");

        let result = loaded(&contents);

        assert!(
            matches!(
                result,
                Err(TenantProfileStorageError::Invalid {
                    source: TenantProfileError::InvalidTenant(PartyError::InvalidEmail(_)),
                    ..
                })
            ),
            "got {result:?}"
        );
    }

    #[test]
    fn given_unknown_template_in_file_when_reading_then_value_is_invalid() {
        let contents = SAMPLE_TOML.replace("\"standard\"", "\"fancy\"");

        let result = loaded(&contents);

        assert!(
            matches!(
                &result,
                Err(TenantProfileStorageError::Invalid {
                    id,
                    source: TenantProfileError::UnknownTemplate(_),
                }) if id == "jeanne-martin"
            ),
            "got {result:?}"
        );
    }
}
