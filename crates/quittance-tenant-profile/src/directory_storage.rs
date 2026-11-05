use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::{TenantProfileId, TenantProfileStorage};

const FILE_EXTENSION: &str = "toml";

/// Fiches enregistrées dans un dossier, un fichier `<id>.toml` par fiche. Le dossier est
/// créé au premier enregistrement.
pub struct DirectoryTenantProfileStorage {
    directory: PathBuf,
}

impl DirectoryTenantProfileStorage {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    fn file_path(&self, id: &TenantProfileId) -> PathBuf {
        self.directory.join(format!("{id}.{FILE_EXTENSION}"))
    }

    fn created_directory(&self) -> io::Result<()> {
        fs::create_dir_all(&self.directory).map_err(|error| located(&self.directory, error))
    }
}

/// Nom sans extension de chaque fichier `.toml` parmi les entrées d'un dossier. Une entrée
/// illisible est ignorée : elle n'empêche pas de lister les autres fiches.
fn profile_file_names(
    entries: impl Iterator<Item = io::Result<PathBuf>>,
) -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    for path in entries.flatten() {
        let is_profile_file = path.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == FILE_EXTENSION);
        if !is_profile_file {
            continue;
        }
        if let Some(stem) = path.file_stem() {
            names.push(stem.to_string_lossy().into_owned());
        }
    }
    Ok(names)
}

/// Erreur d'entrée-sortie qui nomme le chemin en cause, en gardant son type.
fn located(path: &Path, error: io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("`{}`: {error}", path.display()))
}

impl TenantProfileStorage for DirectoryTenantProfileStorage {
    fn stored_names(&self) -> io::Result<Vec<String>> {
        let entries = match fs::read_dir(&self.directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(located(&self.directory, error)),
        };
        profile_file_names(entries.map(|entry| entry.map(|entry| entry.path())))
            .map_err(|error| located(&self.directory, error))
    }

    fn read(&self, id: &TenantProfileId) -> io::Result<String> {
        let path = self.file_path(id);
        fs::read_to_string(&path).map_err(|error| located(&path, error))
    }

    fn write_new(&self, id: &TenantProfileId, contents: &str) -> io::Result<()> {
        self.created_directory()?;
        let path = self.file_path(id);
        File::create_new(&path)
            .and_then(|mut file| file.write_all(contents.as_bytes()))
            .map_err(|error| located(&path, error))
    }

    fn overwrite(&self, id: &TenantProfileId, contents: &str) -> io::Result<()> {
        self.created_directory()?;
        let path = self.file_path(id);
        fs::write(&path, contents).map_err(|error| located(&path, error))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::test_support::unique_temp_path;

    const CONTENTS: &str = "tenant_name = \"Jeanne Martin\"\n";

    fn jeanne() -> TenantProfileId {
        TenantProfileId::parse("jeanne-martin").unwrap()
    }

    #[test]
    fn given_missing_directory_when_listing_then_no_profile_is_stored() {
        let storage = DirectoryTenantProfileStorage::new(unique_temp_path("missing"));

        assert_eq!(storage.stored_names().unwrap(), Vec::<String>::new());
    }

    #[test]
    fn given_missing_directory_when_writing_new_profile_then_directory_and_toml_file_are_created() {
        let directory = unique_temp_path("write-new");
        let storage = DirectoryTenantProfileStorage::new(&directory);

        storage.write_new(&jeanne(), CONTENTS).unwrap();

        assert_eq!(
            fs::read_to_string(directory.join("jeanne-martin.toml")).unwrap(),
            CONTENTS
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn given_existing_file_when_writing_new_profile_then_already_exists_and_file_is_kept() {
        let directory = unique_temp_path("write-new-existing");
        let storage = DirectoryTenantProfileStorage::new(&directory);
        storage.write_new(&jeanne(), CONTENTS).unwrap();

        let result = storage.write_new(&jeanne(), "replaced");

        assert_eq!(
            result.unwrap_err().kind(),
            io::ErrorKind::AlreadyExists,
            "write_new must refuse an existing file"
        );
        assert_eq!(storage.read(&jeanne()).unwrap(), CONTENTS);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn given_existing_file_when_overwriting_then_contents_are_replaced() {
        let directory = unique_temp_path("overwrite");
        let storage = DirectoryTenantProfileStorage::new(&directory);
        storage.write_new(&jeanne(), CONTENTS).unwrap();

        storage.overwrite(&jeanne(), "replaced").unwrap();

        assert_eq!(storage.read(&jeanne()).unwrap(), "replaced");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn given_missing_directory_when_overwriting_then_directory_and_file_are_created() {
        let directory = unique_temp_path("overwrite-missing");
        let storage = DirectoryTenantProfileStorage::new(&directory);

        storage.overwrite(&jeanne(), CONTENTS).unwrap();

        assert_eq!(storage.read(&jeanne()).unwrap(), CONTENTS);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn given_missing_file_when_reading_then_error_is_not_found_and_names_the_file() {
        let directory = unique_temp_path("read-missing");
        let storage = DirectoryTenantProfileStorage::new(&directory);

        let error = storage.read(&jeanne()).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(error.to_string().contains("jeanne-martin.toml"), "{error}");
    }

    #[test]
    fn given_directory_with_other_entries_when_listing_then_only_toml_file_names_are_returned() {
        let directory = unique_temp_path("list");
        fs::create_dir_all(directory.join("archive.toml")).unwrap();
        fs::write(directory.join("paul-durand.toml"), CONTENTS).unwrap();
        fs::write(directory.join("Jeanne Martin.toml"), CONTENTS).unwrap();
        fs::write(directory.join("notes.txt"), CONTENTS).unwrap();
        fs::write(directory.join("paul-durand.toml.bak"), CONTENTS).unwrap();
        let storage = DirectoryTenantProfileStorage::new(&directory);

        let mut names = storage.stored_names().unwrap();
        names.sort();

        assert_eq!(names, ["Jeanne Martin", "paul-durand"]);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn given_directory_entry_in_error_when_listing_then_it_is_skipped_and_the_others_are_listed() {
        let directory = unique_temp_path("list-entry-error");
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("paul-durand.toml"), CONTENTS).unwrap();
        let entries = [
            Err(io::Error::from(io::ErrorKind::PermissionDenied)),
            Ok(directory.join("paul-durand.toml")),
        ];

        let names = profile_file_names(entries.into_iter());

        assert_eq!(names.unwrap(), ["paul-durand"]);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn given_directory_path_taken_by_a_file_when_listing_or_writing_then_both_fail() {
        let file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let storage = DirectoryTenantProfileStorage::new(file_path);

        assert!(storage.stored_names().is_err());
        assert!(storage.write_new(&jeanne(), CONTENTS).is_err());
        assert!(storage.overwrite(&jeanne(), CONTENTS).is_err());
    }
}
