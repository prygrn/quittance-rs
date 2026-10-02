use thiserror::Error;
use time::Date;

/// Erreur de validation d'une partie (bailleur ou locataire).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PartyError {
    #[error("party name is missing")]
    MissingName,
    #[error("party address is missing")]
    MissingAddress,
    #[error("party email is missing")]
    MissingEmail,
    #[error("party email `{0}` is not a valid address")]
    InvalidEmail(String),
}

/// Erreur de validation d'une saisie de quittance.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReceiptError {
    #[error("invalid tenant: {0}")]
    InvalidTenant(#[from] PartyError),
    #[error("property address is missing")]
    MissingPropertyAddress,
    #[error("rent period starts on {start} after it ends on {end}")]
    InvertedPeriod { start: Date, end: Date },
    #[error("total of rent ({rent_cents} cents) and charges ({charges_cents} cents) overflows")]
    TotalOverflow { rent_cents: u64, charges_cents: u64 },
}
