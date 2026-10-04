use std::io;
use std::path::PathBuf;

/// Dossier `tmp/` à la racine du dépôt, ignoré par git.
#[cfg_attr(not(debug_assertions), allow(dead_code))]
const DEBUG_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../tmp");

/// Conservation du PDF de chaque email envoyé, derrière un trait pour tester l'envoi avec
/// des fakes.
pub trait SentPdfArchive {
    fn store(&self, file_name: &str, pdf: &[u8]) -> io::Result<()>;
}

/// Écrit chaque PDF dans un dossier, préfixé par un horodatage pour ne jamais écraser
/// celui d'un envoi précédent sur la même période.
#[cfg_attr(not(debug_assertions), allow(dead_code))]
pub struct DirectoryPdfArchive {
    directory: PathBuf,
}

#[cfg_attr(not(debug_assertions), allow(dead_code))]
impl DirectoryPdfArchive {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }
}

impl SentPdfArchive for DirectoryPdfArchive {
    fn store(&self, file_name: &str, pdf: &[u8]) -> io::Result<()> {
        let _ = (file_name, pdf);
        todo!()
    }
}

/// Ne conserve rien : les PDF ne sont gardés qu'en build debug, pour inspection.
#[cfg_attr(debug_assertions, allow(dead_code))]
pub struct DiscardingPdfArchive;

impl SentPdfArchive for DiscardingPdfArchive {
    fn store(&self, _file_name: &str, _pdf: &[u8]) -> io::Result<()> {
        Ok(())
    }
}

/// En build debug, les PDF envoyés sont écrits dans `tmp/` ; en release, ils ne sont
/// conservés que par la copie cachée reçue par le bailleur.
pub fn sent_pdf_archive_for_build() -> Box<dyn SentPdfArchive> {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    const PDF_CONTENT: &[u8] = b"%PDF-1.7 quittance";

    /// Dossier propre au test, absent au départ.
    fn unique_test_directory(test_name: &str) -> PathBuf {
        let nanoseconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "quittance-app-{test_name}-{}-{nanoseconds}",
            process::id()
        ))
    }

    fn stored_files(directory: &Path) -> Vec<PathBuf> {
        fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect()
    }

    #[test]
    fn given_missing_directory_when_storing_pdf_then_directory_is_created_with_the_pdf() {
        let directory = unique_test_directory("store");
        let archive = DirectoryPdfArchive::new(&directory);

        archive.store("quittance-2026-10.pdf", PDF_CONTENT).unwrap();

        let files = stored_files(&directory);
        assert_eq!(files.len(), 1);
        let file_name = files[0].file_name().unwrap().to_string_lossy().into_owned();
        assert!(file_name.ends_with("quittance-2026-10.pdf"), "{file_name}");
        assert_eq!(fs::read(&files[0]).unwrap(), PDF_CONTENT);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn given_two_pdfs_of_the_same_period_when_storing_then_both_are_kept() {
        let directory = unique_test_directory("store-twice");
        let archive = DirectoryPdfArchive::new(&directory);

        archive.store("quittance-2026-10.pdf", PDF_CONTENT).unwrap();
        archive.store("quittance-2026-10.pdf", PDF_CONTENT).unwrap();

        assert_eq!(stored_files(&directory).len(), 2);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn given_directory_path_taken_by_a_file_when_storing_pdf_then_storing_fails() {
        let file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let archive = DirectoryPdfArchive::new(file_path);

        let result = archive.store("quittance-2026-10.pdf", PDF_CONTENT);

        assert!(result.is_err());
    }

    #[test]
    fn given_debug_directory_when_inspecting_then_it_is_tmp_at_the_repository_root() {
        let directory = Path::new(DEBUG_DIRECTORY);

        assert_eq!(directory.file_name().unwrap(), "tmp");
        let repository_root = directory.parent().unwrap();
        assert!(repository_root.join(".gitignore").is_file());
        assert!(repository_root.join("src-tauri").is_dir());
    }

    #[test]
    fn given_discarding_archive_when_storing_pdf_then_storing_succeeds() {
        let result = DiscardingPdfArchive.store("quittance-2026-10.pdf", PDF_CONTENT);

        assert!(result.is_ok());
    }
}
