use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{TenantProfileId, TenantProfileStorage};

const FILE_EXTENSION: &str = "toml";
/// Extension des fichiers en cours d'écriture : un fichier abandonné n'est jamais listé.
const TEMPORARY_FILE_EXTENSION: &str = "tmp";

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

    /// Écrit la fiche dans un fichier temporaire du dossier, puis `place` le met à la place
    /// du fichier de la fiche : un arrêt en cours d'écriture ne laisse jamais de fiche
    /// tronquée, et un lien symbolique posé à la place de la fiche n'est jamais suivi.
    fn write_through_temporary_file(
        &self,
        id: &TenantProfileId,
        contents: &str,
        place: impl FnOnce(&Path, &Path) -> io::Result<()>,
    ) -> io::Result<()> {
        self.created_directory()?;
        let path = self.file_path(id);
        let temporary_path = self.temporary_file_path(id)?;
        let result = synced_new_file(&temporary_path, contents)
            .and_then(|()| place(&temporary_path, &path))
            .map_err(|error| located(&path, error));
        // Nettoyage au mieux : après un renommage le fichier n'existe plus, et un fichier
        // temporaire restant n'est jamais lu comme une fiche.
        let _ = fs::remove_file(&temporary_path);
        result
    }

    fn temporary_file_path(&self, id: &TenantProfileId) -> io::Result<PathBuf> {
        let nanoseconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        Ok(self.directory.join(format!(
            ".{id}.{FILE_EXTENSION}.{}-{nanoseconds}.{TEMPORARY_FILE_EXTENSION}",
            process::id()
        )))
    }
}

/// Crée le fichier, qui ne doit pas exister, et attend que son contenu soit sur le disque.
fn synced_new_file(path: &Path, contents: &str) -> io::Result<()> {
    let mut file = File::create_new(path)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()
}

/// Met le fichier temporaire à la place de la fiche par un lien physique, qui échoue si la
/// fiche existe déjà, contrairement à un renommage.
fn placed_without_overwrite(
    temporary_path: &Path,
    path: &Path,
    contents: &str,
    hard_link: impl FnOnce(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    let _ = contents;
    hard_link(temporary_path, path)
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
        self.write_through_temporary_file(id, contents, |temporary_path, path| {
            placed_without_overwrite(temporary_path, path, contents, |from, to| {
                fs::hard_link(from, to)
            })
        })
    }

    fn overwrite(&self, id: &TenantProfileId, contents: &str) -> io::Result<()> {
        self.write_through_temporary_file(id, contents, |temporary_path, path| {
            fs::rename(temporary_path, path)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::test_fixtures::unique_temp_path;

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
    fn given_profiles_written_when_listing_the_directory_then_no_temporary_file_is_left() {
        let directory = unique_temp_path("no-temporary-file");
        let storage = DirectoryTenantProfileStorage::new(&directory);

        storage.write_new(&jeanne(), CONTENTS).unwrap();
        storage.overwrite(&jeanne(), "replaced").unwrap();
        let _ = storage.write_new(&jeanne(), "refused");

        let file_names: Vec<_> = fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(file_names, ["jeanne-martin.toml"]);
        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn given_symbolic_link_in_place_of_the_profile_when_overwriting_then_link_target_is_untouched()
    {
        let directory = unique_temp_path("overwrite-symlink");
        let outside = unique_temp_path("overwrite-symlink-target");
        fs::create_dir_all(&directory).unwrap();
        fs::write(&outside, "outside").unwrap();
        std::os::unix::fs::symlink(&outside, directory.join("jeanne-martin.toml")).unwrap();
        let storage = DirectoryTenantProfileStorage::new(&directory);

        storage.overwrite(&jeanne(), CONTENTS).unwrap();

        assert_eq!(fs::read_to_string(&outside).unwrap(), "outside");
        let profile_path = directory.join("jeanne-martin.toml");
        assert!(!profile_path.symlink_metadata().unwrap().is_symlink());
        assert_eq!(fs::read_to_string(profile_path).unwrap(), CONTENTS);
        fs::remove_dir_all(directory).unwrap();
        fs::remove_file(outside).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn given_dangling_symbolic_link_in_place_of_the_profile_when_writing_new_then_it_already_exists()
     {
        let directory = unique_temp_path("write-new-symlink");
        let outside = unique_temp_path("write-new-symlink-target");
        fs::create_dir_all(&directory).unwrap();
        std::os::unix::fs::symlink(&outside, directory.join("jeanne-martin.toml")).unwrap();
        let storage = DirectoryTenantProfileStorage::new(&directory);

        let result = storage.write_new(&jeanne(), CONTENTS);

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert!(!outside.exists());
        fs::remove_dir_all(directory).unwrap();
    }

    fn unsupported_hard_link(_: &Path, _: &Path) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    #[test]
    fn given_file_system_without_hard_links_when_placing_new_profile_then_it_is_created() {
        let directory = unique_temp_path("place-without-hard-link");
        fs::create_dir_all(&directory).unwrap();
        let temporary_path = directory.join(".jeanne-martin.toml.tmp");
        let path = directory.join("jeanne-martin.toml");
        fs::write(&temporary_path, CONTENTS).unwrap();

        placed_without_overwrite(&temporary_path, &path, CONTENTS, unsupported_hard_link).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), CONTENTS);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn given_file_system_without_hard_links_when_placing_existing_profile_then_it_already_exists() {
        let directory = unique_temp_path("place-existing-without-hard-link");
        fs::create_dir_all(&directory).unwrap();
        let temporary_path = directory.join(".jeanne-martin.toml.tmp");
        let path = directory.join("jeanne-martin.toml");
        fs::write(&temporary_path, "replaced").unwrap();
        fs::write(&path, CONTENTS).unwrap();

        let result =
            placed_without_overwrite(&temporary_path, &path, "replaced", unsupported_hard_link);

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(&path).unwrap(), CONTENTS);
        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn given_directory_where_no_file_can_be_created_when_saving_then_error_names_the_temporary_file()
     {
        use std::os::unix::fs::PermissionsExt;

        let directory = unique_temp_path("read-only-directory");
        fs::create_dir_all(&directory).unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o555)).unwrap();
        let storage = DirectoryTenantProfileStorage::new(&directory);

        let errors = [
            storage.write_new(&jeanne(), CONTENTS).unwrap_err(),
            storage.overwrite(&jeanne(), CONTENTS).unwrap_err(),
        ];

        fs::set_permissions(&directory, fs::Permissions::from_mode(0o755)).unwrap();
        fs::remove_dir_all(&directory).unwrap();
        for error in errors {
            let message = error.to_string();
            assert!(message.contains(".jeanne-martin.toml."), "{message}");
            assert!(message.contains(".tmp`"), "{message}");
        }
    }

    #[test]
    fn given_existing_profile_when_writing_new_then_error_names_the_profile_file() {
        let directory = unique_temp_path("write-new-existing-message");
        let storage = DirectoryTenantProfileStorage::new(&directory);
        storage.write_new(&jeanne(), CONTENTS).unwrap();

        let message = storage
            .write_new(&jeanne(), CONTENTS)
            .unwrap_err()
            .to_string();

        assert!(message.contains("jeanne-martin.toml`"), "{message}");
        assert!(!message.contains(".tmp"), "{message}");
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
