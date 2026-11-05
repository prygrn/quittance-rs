use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{TenantProfile, TenantProfileFields, TenantProfileId, TenantProfileStorage};

pub(crate) const KNOWN_TEMPLATE_IDS: &[&str] = &["standard"];

/// Fichier de la fiche de [`sample_fields`], au format du README.
pub(crate) const SAMPLE_TOML: &str = r#"template_id = "standard"
tenant_name = "Jeanne Martin"
tenant_address = "8 rue Victor Hugo, 75011 Paris"
tenant_email = "jeanne.martin@example.fr"
property_address = "12 rue des Lilas, 75011 Paris"
rent = "650,00"
charges = "50,50"
"#;

pub(crate) fn sample_fields() -> TenantProfileFields {
    TenantProfileFields {
        template_id: "standard".to_owned(),
        tenant_name: "Jeanne Martin".to_owned(),
        tenant_address: "8 rue Victor Hugo, 75011 Paris".to_owned(),
        tenant_email: "jeanne.martin@example.fr".to_owned(),
        property_address: "12 rue des Lilas, 75011 Paris".to_owned(),
        rent_cents: 65_000,
        charges_cents: 5_050,
    }
}

pub(crate) fn sample_profile() -> TenantProfile {
    TenantProfile::new(sample_fields(), KNOWN_TEMPLATE_IDS).unwrap()
}

pub(crate) fn profile_named(tenant_name: &str) -> TenantProfile {
    let fields = TenantProfileFields {
        tenant_name: tenant_name.to_owned(),
        ..sample_fields()
    };
    TenantProfile::new(fields, KNOWN_TEMPLATE_IDS).unwrap()
}

/// Chemin propre à un test dans le dossier temporaire du système, absent au départ.
pub(crate) fn unique_temp_path(test_name: &str) -> PathBuf {
    let nanoseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "quittance-tenant-profile-{test_name}-{}-{nanoseconds}",
        process::id()
    ))
}

/// Fichiers de fiche en mémoire, indexés par nom sans extension, avec échecs programmables.
#[derive(Default)]
pub(crate) struct FakeTenantProfileStorage {
    pub files: RefCell<BTreeMap<String, String>>,
    pub listing_failure: Option<fn() -> io::Error>,
    pub read_failure: Option<fn() -> io::Error>,
    pub write_failure: Option<fn() -> io::Error>,
    pub reads: RefCell<Vec<String>>,
}

impl FakeTenantProfileStorage {
    pub(crate) fn with_files<const N: usize>(files: [(&str, String); N]) -> Self {
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

    pub(crate) fn contents(&self, name: &str) -> Option<String> {
        self.files.borrow().get(name).cloned()
    }

    /// Ids lus, dans l'ordre des lectures.
    pub(crate) fn read_ids(&self) -> Vec<String> {
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
