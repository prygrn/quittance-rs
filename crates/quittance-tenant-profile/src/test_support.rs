//! Doublure en mémoire de [`TenantProfileStorage`], pour tester les appelants sans disque.
//! Exposée hors du crate par la feature `test-support`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;

use crate::{TenantProfileId, TenantProfileStorage};

/// Fichiers de fiche en mémoire, indexés par nom sans extension, avec échecs programmables.
#[derive(Default)]
pub struct FakeTenantProfileStorage {
    pub files: RefCell<BTreeMap<String, String>>,
    pub listing_failure: Option<fn() -> io::Error>,
    pub read_failure: Option<fn() -> io::Error>,
    pub write_failure: Option<fn() -> io::Error>,
    pub reads: RefCell<Vec<String>>,
}

impl FakeTenantProfileStorage {
    pub fn with_files<const N: usize>(files: [(&str, String); N]) -> Self {
        Self {
            files: RefCell::new(
                files
                    .into_iter()
                    .map(|(name, contents)| (name.to_owned(), contents))
                    .collect(),
            ),
            ..Self::default()
        }
    }

    pub fn contents(&self, name: &str) -> Option<String> {
        self.files.borrow().get(name).cloned()
    }

    /// Ids lus, dans l'ordre des lectures.
    pub fn read_ids(&self) -> Vec<String> {
        self.reads.borrow().clone()
    }
}

impl TenantProfileStorage for FakeTenantProfileStorage {
    fn stored_names(&self) -> io::Result<Vec<String>> {
        if let Some(failure) = self.listing_failure {
            return Err(failure());
        }
        Ok(self.files.borrow().keys().cloned().collect())
    }

    fn read(&self, id: &TenantProfileId) -> io::Result<String> {
        self.reads.borrow_mut().push(id.as_str().to_owned());
        if let Some(failure) = self.read_failure {
            return Err(failure());
        }
        self.contents(id.as_str())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }

    fn write_new(&self, id: &TenantProfileId, contents: &str) -> io::Result<()> {
        if self.files.borrow().contains_key(id.as_str()) {
            return Err(io::Error::from(io::ErrorKind::AlreadyExists));
        }
        self.overwrite(id, contents)
    }

    fn overwrite(&self, id: &TenantProfileId, contents: &str) -> io::Result<()> {
        if let Some(failure) = self.write_failure {
            return Err(failure());
        }
        self.files
            .borrow_mut()
            .insert(id.as_str().to_owned(), contents.to_owned());
        Ok(())
    }
}
