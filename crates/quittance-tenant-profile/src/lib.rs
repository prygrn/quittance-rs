//! Fiche locataire : tout ce qui préremplit une quittance, sauf la période et la date de
//! paiement. Chaque fiche est un fichier TOML `<id>.toml`, dont l'id dérive du nom du
//! locataire.

mod directory_storage;
mod error;
mod french_amount;
mod profile_file;
mod repository;
mod storage;
mod tenant_profile;
mod tenant_profile_id;

pub use directory_storage::DirectoryTenantProfileStorage;
pub use error::{TenantProfileError, TenantProfileStorageError};
pub use repository::{TenantProfileEntry, TenantProfileRepository};
pub use storage::TenantProfileStorage;
pub use tenant_profile::{TenantProfile, TenantProfileFields};
pub use tenant_profile_id::TenantProfileId;

#[cfg(test)]
mod test_support;
