use std::io;

use crate::profile_file::{profile_from_toml, profile_toml};
use crate::{TenantProfile, TenantProfileId, TenantProfileStorage, TenantProfileStorageError};

/// Fiche proposée à l'UI : `label` est le nom du locataire, ou l'id si la fiche ne peut
/// pas être chargée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantProfileEntry {
    pub id: String,
    pub label: String,
}

/// Lecture et enregistrement des fiches. Une fiche chargée est validée comme à
/// l'enregistrement, contre les mêmes templates connus.
pub struct TenantProfileRepository<'a> {
    pub storage: &'a dyn TenantProfileStorage,
    pub known_template_ids: &'a [&'a str],
}

impl TenantProfileRepository<'_> {
    /// Une entrée par fichier de fiche, triée par libellé. Une fiche illisible ou invalide
    /// n'empêche pas de lister les autres.
    pub fn list(&self) -> Result<Vec<TenantProfileEntry>, TenantProfileStorageError> {
        let names = self
            .storage
            .stored_names()
            .map_err(TenantProfileStorageError::Unlistable)?;
        let mut entries: Vec<TenantProfileEntry> = names
            .into_iter()
            .map(|name| TenantProfileEntry {
                label: self.load(&name).map_or_else(
                    |_| name.clone(),
                    |profile| profile.tenant().name().to_owned(),
                ),
                id: name,
            })
            .collect();
        entries.sort_by(|left, right| (&left.label, &left.id).cmp(&(&right.label, &right.id)));
        Ok(entries)
    }

    pub fn load(&self, id: &str) -> Result<TenantProfile, TenantProfileStorageError> {
        let id = TenantProfileId::parse(id)?;
        let contents = self.storage.read(&id).map_err(|source| {
            if source.kind() == io::ErrorKind::NotFound {
                TenantProfileStorageError::NotFound { id: id.to_string() }
            } else {
                TenantProfileStorageError::Unreadable {
                    id: id.to_string(),
                    source,
                }
            }
        })?;
        profile_from_toml(&id, &contents, self.known_template_ids)
    }

    /// Enregistre une nouvelle fiche ; refuse d'écraser une fiche de même id.
    pub fn create(
        &self,
        profile: &TenantProfile,
    ) -> Result<TenantProfileEntry, TenantProfileStorageError> {
        let contents = file_contents(profile)?;
        self.storage
            .write_new(profile.id(), &contents)
            .map_err(|source| {
                if source.kind() == io::ErrorKind::AlreadyExists {
                    TenantProfileStorageError::AlreadyExists {
                        id: profile.id().to_string(),
                    }
                } else {
                    unwritable(profile, source)
                }
            })?;
        Ok(entry_of(profile))
    }

    /// Enregistre la fiche, en écrasant celle de même id si elle existe.
    pub fn replace(
        &self,
        profile: &TenantProfile,
    ) -> Result<TenantProfileEntry, TenantProfileStorageError> {
        let contents = file_contents(profile)?;
        self.storage
            .overwrite(profile.id(), &contents)
            .map_err(|source| unwritable(profile, source))?;
        Ok(entry_of(profile))
    }
}

fn file_contents(profile: &TenantProfile) -> Result<String, TenantProfileStorageError> {
    profile_toml(profile).map_err(|error| unwritable(profile, io::Error::other(error)))
}

fn unwritable(profile: &TenantProfile, source: io::Error) -> TenantProfileStorageError {
    TenantProfileStorageError::Unwritable {
        id: profile.id().to_string(),
        source,
    }
}

fn entry_of(profile: &TenantProfile) -> TenantProfileEntry {
    TenantProfileEntry {
        id: profile.id().to_string(),
        label: profile.tenant().name().to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;
    use crate::test_support::{
        FakeTenantProfileStorage, KNOWN_TEMPLATE_IDS, SAMPLE_TOML, profile_named, sample_profile,
    };

    fn repository(storage: &FakeTenantProfileStorage) -> TenantProfileRepository<'_> {
        TenantProfileRepository {
            storage,
            known_template_ids: KNOWN_TEMPLATE_IDS,
        }
    }

    fn entry(id: &str, label: &str) -> TenantProfileEntry {
        TenantProfileEntry {
            id: id.to_owned(),
            label: label.to_owned(),
        }
    }

    #[test]
    fn given_no_stored_profile_when_listing_then_list_is_empty() {
        let storage = FakeTenantProfileStorage::default();

        assert_eq!(repository(&storage).list().unwrap(), []);
    }

    #[test]
    fn given_stored_profiles_when_listing_then_entries_carry_tenant_names_sorted_by_label() {
        let storage = FakeTenantProfileStorage::default();
        repository(&storage)
            .create(&profile_named("Paul Durand"))
            .unwrap();
        repository(&storage)
            .create(&profile_named("Jeanne Martin"))
            .unwrap();

        let entries = repository(&storage).list().unwrap();

        assert_eq!(
            entries,
            [
                entry("jeanne-martin", "Jeanne Martin"),
                entry("paul-durand", "Paul Durand")
            ]
        );
    }

    #[test]
    fn given_unloadable_files_when_listing_then_they_are_listed_by_id_among_the_others() {
        let storage = FakeTenantProfileStorage::with_files([
            (
                "zoe-bernard",
                SAMPLE_TOML.replace("Jeanne Martin", "Zoé Bernard"),
            ),
            ("broken", "tenant_name = ".to_owned()),
            ("Notes Perso", SAMPLE_TOML.to_owned()),
        ]);

        let entries = repository(&storage).list().unwrap();

        assert_eq!(
            entries,
            [
                entry("Notes Perso", "Notes Perso"),
                entry("zoe-bernard", "Zoé Bernard"),
                entry("broken", "broken")
            ]
        );
    }

    #[test]
    fn given_storage_that_cannot_be_listed_when_listing_then_it_is_unlistable() {
        let storage = FakeTenantProfileStorage {
            listing_failure: Some(|| io::Error::from(io::ErrorKind::PermissionDenied)),
            ..FakeTenantProfileStorage::default()
        };

        let result = repository(&storage).list();

        assert!(
            matches!(result, Err(TenantProfileStorageError::Unlistable(_))),
            "got {result:?}"
        );
    }

    #[test]
    fn given_created_profile_when_loading_its_id_then_same_profile_is_returned() {
        let storage = FakeTenantProfileStorage::default();
        repository(&storage).create(&sample_profile()).unwrap();

        let profile = repository(&storage).load("jeanne-martin").unwrap();

        assert_eq!(profile, sample_profile());
    }

    #[test]
    fn given_missing_profile_when_loading_then_it_is_not_found() {
        let storage = FakeTenantProfileStorage::default();

        let result = repository(&storage).load("jeanne-martin");

        assert!(
            matches!(&result, Err(TenantProfileStorageError::NotFound { id }) if id == "jeanne-martin"),
            "got {result:?}"
        );
    }

    #[test]
    fn given_id_that_is_not_a_profile_id_when_loading_then_storage_is_never_read() {
        let storage =
            FakeTenantProfileStorage::with_files([("Notes Perso", SAMPLE_TOML.to_owned())]);

        for id in ["../.env", "Notes Perso"] {
            let result = repository(&storage).load(id);

            assert!(
                matches!(&result, Err(TenantProfileStorageError::InvalidId(rejected)) if rejected == id),
                "{id:?} got {result:?}"
            );
        }
        assert_eq!(storage.read_ids(), Vec::<String>::new());
    }

    #[test]
    fn given_unreadable_file_when_loading_then_it_is_unreadable() {
        let storage = FakeTenantProfileStorage {
            read_failure: Some(|| io::Error::from(io::ErrorKind::InvalidData)),
            ..FakeTenantProfileStorage::with_files([("jeanne-martin", SAMPLE_TOML.to_owned())])
        };

        let result = repository(&storage).load("jeanne-martin");

        assert!(
            matches!(&result, Err(TenantProfileStorageError::Unreadable { id, .. }) if id == "jeanne-martin"),
            "got {result:?}"
        );
    }

    #[test]
    fn given_file_with_unknown_key_when_loading_then_it_is_malformed() {
        let storage = FakeTenantProfileStorage::with_files([(
            "jeanne-martin",
            format!("{SAMPLE_TOML}deposit = \"1300,00\"\n"),
        )]);

        let result = repository(&storage).load("jeanne-martin");

        assert!(
            matches!(result, Err(TenantProfileStorageError::Malformed { .. })),
            "got {result:?}"
        );
    }

    #[test]
    fn given_file_with_template_outside_the_known_ones_when_loading_then_it_is_invalid() {
        let storage =
            FakeTenantProfileStorage::with_files([("jeanne-martin", SAMPLE_TOML.to_owned())]);
        let repository = TenantProfileRepository {
            storage: &storage,
            known_template_ids: &["fancy"],
        };

        let result = repository.load("jeanne-martin");

        assert!(
            matches!(result, Err(TenantProfileStorageError::Invalid { .. })),
            "got {result:?}"
        );
    }

    #[test]
    fn given_new_profile_when_creating_then_its_file_is_written_and_its_entry_returned() {
        let storage = FakeTenantProfileStorage::default();

        let created = repository(&storage).create(&sample_profile()).unwrap();

        assert_eq!(created, entry("jeanne-martin", "Jeanne Martin"));
        assert_eq!(
            storage.contents("jeanne-martin").as_deref(),
            Some(SAMPLE_TOML)
        );
    }

    #[test]
    fn given_profile_with_the_same_id_when_creating_then_it_already_exists_and_file_is_kept() {
        let storage = FakeTenantProfileStorage::default();
        repository(&storage).create(&sample_profile()).unwrap();
        let homonym = profile_named("JEANNE MARTIN");

        let result = repository(&storage).create(&homonym);

        assert!(
            matches!(&result, Err(TenantProfileStorageError::AlreadyExists { id }) if id == "jeanne-martin"),
            "got {result:?}"
        );
        assert_eq!(
            storage.contents("jeanne-martin").as_deref(),
            Some(SAMPLE_TOML)
        );
    }

    #[test]
    fn given_existing_profile_when_replacing_then_its_file_is_overwritten() {
        let storage = FakeTenantProfileStorage::default();
        repository(&storage).create(&sample_profile()).unwrap();
        let homonym = profile_named("JEANNE MARTIN");

        let replaced = repository(&storage).replace(&homonym).unwrap();

        assert_eq!(replaced, entry("jeanne-martin", "JEANNE MARTIN"));
        assert_eq!(repository(&storage).load("jeanne-martin").unwrap(), homonym);
    }

    #[test]
    fn given_missing_profile_when_replacing_then_it_is_written() {
        let storage = FakeTenantProfileStorage::default();

        repository(&storage).replace(&sample_profile()).unwrap();

        assert_eq!(
            storage.contents("jeanne-martin").as_deref(),
            Some(SAMPLE_TOML)
        );
    }

    #[test]
    fn given_storage_that_cannot_be_written_when_saving_then_profile_is_unwritable() {
        let storage = FakeTenantProfileStorage {
            write_failure: Some(|| io::Error::from(io::ErrorKind::PermissionDenied)),
            ..FakeTenantProfileStorage::default()
        };

        let results = [
            repository(&storage).create(&sample_profile()),
            repository(&storage).replace(&sample_profile()),
        ];

        for result in results {
            assert!(
                matches!(&result, Err(TenantProfileStorageError::Unwritable { id, .. }) if id == "jeanne-martin"),
                "got {result:?}"
            );
        }
    }
}
