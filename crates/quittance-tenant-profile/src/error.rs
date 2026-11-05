use std::io;

use quittance_core::PartyError;
use thiserror::Error;

/// Contenu de fiche invalide, à l'enregistrement comme au chargement.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TenantProfileError {
    #[error("template `{0}` does not exist")]
    UnknownTemplate(String),
    #[error("invalid tenant: {0}")]
    InvalidTenant(#[from] PartyError),
    #[error("property address is missing")]
    MissingPropertyAddress,
    #[error("total of rent ({rent_cents} cents) and charges ({charges_cents} cents) overflows")]
    TotalOverflow { rent_cents: u64, charges_cents: u64 },
    #[error("tenant name `{tenant_name}` yields an empty tenant profile id")]
    EmptyId { tenant_name: String },
    #[error("`{field}` amount `{value}` is not a euro amount such as `650,00`")]
    InvalidAmount { field: &'static str, value: String },
}

/// Échec de lecture ou d'écriture d'une fiche enregistrée.
#[derive(Debug, Error)]
pub enum TenantProfileStorageError {
    #[error("`{0}` is not a tenant profile id")]
    InvalidId(String),
    #[error("tenant profile `{id}` does not exist")]
    NotFound { id: String },
    #[error("tenant profile `{id}` already exists")]
    AlreadyExists { id: String },
    #[error("tenant profiles cannot be listed: {0}")]
    Unlistable(#[source] io::Error),
    #[error("tenant profile `{id}` cannot be read: {source}")]
    Unreadable {
        id: String,
        #[source]
        source: io::Error,
    },
    #[error("tenant profile `{id}` is not a valid profile file: {source}")]
    Malformed {
        id: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("tenant profile `{id}` holds an invalid value: {source}")]
    Invalid {
        id: String,
        #[source]
        source: TenantProfileError,
    },
    #[error("tenant profile `{id}` cannot be written: {source}")]
    Unwritable {
        id: String,
        #[source]
        source: io::Error,
    },
}
