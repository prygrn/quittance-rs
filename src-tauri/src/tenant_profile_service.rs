use quittance_tenant_profile::{TenantProfile, TenantProfileRepository, TenantProfileStorage};

use crate::command_error::CommandError;
use crate::tenant_profile_entry_payload::TenantProfileEntryPayload;
use crate::tenant_profile_payload::TenantProfilePayload;

/// Templates de quittance qu'une fiche peut désigner.
pub(crate) fn known_template_ids() -> Vec<&'static str> {
    quittance_template::list_templates()
        .into_iter()
        .map(|template| template.id)
        .collect()
}

/// Orchestration des commandes de fiche locataire. Une fiche ne peut désigner qu'un template
/// de `quittance-template`, à l'enregistrement comme au chargement.
pub struct TenantProfileService<'a> {
    pub storage: &'a dyn TenantProfileStorage,
}

impl TenantProfileService<'_> {
    /// Fiches enregistrées, triées par libellé.
    pub fn list(&self) -> Result<Vec<TenantProfileEntryPayload>, CommandError> {
        let entries = self.with_repository(|repository| repository.list())?;
        Ok(entries
            .into_iter()
            .map(TenantProfileEntryPayload::from)
            .collect())
    }

    pub fn load(&self, id: &str) -> Result<TenantProfilePayload, CommandError> {
        let profile = self.with_repository(|repository| repository.load(id))?;
        Ok(TenantProfilePayload::from(&profile))
    }

    /// Enregistre une nouvelle fiche ; `tenantProfileExists` si une fiche de même id existe.
    pub fn create(
        &self,
        profile: TenantProfilePayload,
    ) -> Result<TenantProfileEntryPayload, CommandError> {
        let profile = validated(profile)?;
        let entry = self.with_repository(|repository| repository.create(&profile))?;
        Ok(TenantProfileEntryPayload::from(entry))
    }

    /// Enregistre la fiche, en écrasant celle de même id.
    pub fn replace(
        &self,
        profile: TenantProfilePayload,
    ) -> Result<TenantProfileEntryPayload, CommandError> {
        let profile = validated(profile)?;
        let entry = self.with_repository(|repository| repository.replace(&profile))?;
        Ok(TenantProfileEntryPayload::from(entry))
    }

    fn with_repository<T, Error>(
        &self,
        action: impl FnOnce(&TenantProfileRepository<'_>) -> Result<T, Error>,
    ) -> Result<T, CommandError>
    where
        CommandError: From<Error>,
    {
        let known_template_ids = known_template_ids();
        let repository = TenantProfileRepository {
            storage: self.storage,
            known_template_ids: &known_template_ids,
        };
        Ok(action(&repository)?)
    }
}

fn validated(profile: TenantProfilePayload) -> Result<TenantProfile, CommandError> {
    Ok(TenantProfile::new(
        profile.into_fields(),
        &known_template_ids(),
    )?)
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;
    use crate::command_error::CommandErrorCode;
    use crate::test_support::{FakeTenantProfileStorage, sample_tenant_profile_payload};

    fn service(storage: &FakeTenantProfileStorage) -> TenantProfileService<'_> {
        TenantProfileService { storage }
    }

    fn entry(id: &str, label: &str) -> TenantProfileEntryPayload {
        TenantProfileEntryPayload {
            id: id.to_owned(),
            label: label.to_owned(),
        }
    }

    fn payload_named(tenant_name: &str) -> TenantProfilePayload {
        TenantProfilePayload {
            tenant_name: tenant_name.to_owned(),
            ..sample_tenant_profile_payload()
        }
    }

    fn code_of<T: std::fmt::Debug>(result: Result<T, CommandError>) -> CommandErrorCode {
        result.unwrap_err().code()
    }

    #[test]
    fn given_new_profile_when_creating_then_its_entry_is_returned_and_it_loads_back() {
        let storage = FakeTenantProfileStorage::default();

        let created = service(&storage)
            .create(sample_tenant_profile_payload())
            .unwrap();

        assert_eq!(created, entry("jeanne-martin", "Jeanne Martin"));
        assert_eq!(
            service(&storage).load("jeanne-martin").unwrap(),
            sample_tenant_profile_payload()
        );
    }

    #[test]
    fn given_invalid_profile_when_creating_then_code_is_validation_and_nothing_is_stored() {
        let invalid_payloads = [
            TenantProfilePayload {
                tenant_email: "jeanne.martin".to_owned(),
                ..sample_tenant_profile_payload()
            },
            TenantProfilePayload {
                property_address: " ".to_owned(),
                ..sample_tenant_profile_payload()
            },
            TenantProfilePayload {
                template_id: "fancy".to_owned(),
                ..sample_tenant_profile_payload()
            },
            TenantProfilePayload {
                rent_cents: u64::MAX,
                ..sample_tenant_profile_payload()
            },
            payload_named("!!!"),
        ];

        for payload in invalid_payloads {
            let storage = FakeTenantProfileStorage::default();

            let code = code_of(service(&storage).create(payload.clone()));

            assert_eq!(code, CommandErrorCode::Validation, "{payload:?}");
            assert_eq!(storage.stored_file_count(), 0, "{payload:?}");
        }
    }

    #[test]
    fn given_existing_profile_when_creating_the_same_id_then_code_is_tenant_profile_exists() {
        let storage = FakeTenantProfileStorage::default();
        service(&storage)
            .create(sample_tenant_profile_payload())
            .unwrap();

        let code = code_of(service(&storage).create(payload_named("JEANNE MARTIN")));

        assert_eq!(code, CommandErrorCode::TenantProfileExists);
        assert_eq!(
            service(&storage).load("jeanne-martin").unwrap(),
            sample_tenant_profile_payload()
        );
    }

    #[test]
    fn given_existing_profile_when_replacing_then_new_values_load_back() {
        let storage = FakeTenantProfileStorage::default();
        service(&storage)
            .create(sample_tenant_profile_payload())
            .unwrap();
        let replacement = TenantProfilePayload {
            rent_cents: 70_000,
            ..payload_named("JEANNE MARTIN")
        };

        let replaced = service(&storage).replace(replacement.clone()).unwrap();

        assert_eq!(replaced, entry("jeanne-martin", "JEANNE MARTIN"));
        assert_eq!(
            service(&storage).load("jeanne-martin").unwrap(),
            replacement
        );
    }

    #[test]
    fn given_invalid_profile_when_replacing_then_code_is_validation() {
        let storage = FakeTenantProfileStorage::default();
        let payload = TenantProfilePayload {
            template_id: "fancy".to_owned(),
            ..sample_tenant_profile_payload()
        };

        let code = code_of(service(&storage).replace(payload));

        assert_eq!(code, CommandErrorCode::Validation);
        assert_eq!(storage.stored_file_count(), 0);
    }

    #[test]
    fn given_profile_that_cannot_be_loaded_when_loading_then_code_is_tenant_profile() {
        let storage = FakeTenantProfileStorage::with_file(
            "jeanne-martin",
            "template_id = \"retired-template\"\n",
        );

        for id in ["jeanne-martin", "paul-durand", "../.env"] {
            let code = code_of(service(&storage).load(id));

            assert_eq!(code, CommandErrorCode::TenantProfile, "{id}");
        }
    }

    #[test]
    fn given_stored_profile_with_template_no_longer_available_when_loading_then_code_is_tenant_profile()
     {
        let storage = FakeTenantProfileStorage::default();
        service(&storage)
            .create(sample_tenant_profile_payload())
            .unwrap();
        storage.replace_in_file("jeanne-martin", "\"standard\"", "\"retired-template\"");

        let code = code_of(service(&storage).load("jeanne-martin"));

        assert_eq!(code, CommandErrorCode::TenantProfile);
    }

    #[test]
    fn given_stored_profiles_when_listing_then_entries_are_sorted_by_label() {
        let storage = FakeTenantProfileStorage::default();
        service(&storage)
            .create(payload_named("Paul Durand"))
            .unwrap();
        service(&storage)
            .create(sample_tenant_profile_payload())
            .unwrap();

        let entries = service(&storage).list().unwrap();

        assert_eq!(
            entries,
            [
                entry("jeanne-martin", "Jeanne Martin"),
                entry("paul-durand", "Paul Durand")
            ]
        );
    }

    #[test]
    fn given_storage_failures_when_listing_or_saving_then_code_is_tenant_profile() {
        let storage = FakeTenantProfileStorage {
            failure: Some(|| io::Error::from(io::ErrorKind::PermissionDenied)),
            ..FakeTenantProfileStorage::default()
        };

        assert_eq!(
            code_of(service(&storage).list()),
            CommandErrorCode::TenantProfile
        );
        assert_eq!(
            code_of(service(&storage).create(sample_tenant_profile_payload())),
            CommandErrorCode::TenantProfile
        );
        assert_eq!(
            code_of(service(&storage).replace(sample_tenant_profile_payload())),
            CommandErrorCode::TenantProfile
        );
    }
}
