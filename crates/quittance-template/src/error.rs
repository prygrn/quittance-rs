use std::error::Error as StdError;

use french_amount_words::AmountWordsError;
use thiserror::Error;

/// Échec du rendu HTML d'une quittance.
#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("template `{0}` does not exist")]
    UnknownTemplate(String),
    /// Seul un PNG embarqué en base64 (`data:image/png;base64,…`) est accepté.
    #[error("signature is not a base64 png data URI")]
    InvalidSignature,
    /// Montant total au-delà de [`french_amount_words::MAX_VALUE`] euros.
    #[error("total amount cannot be written in words: {0}")]
    AmountInWords(#[from] AmountWordsError),
    /// Template embarqué incohérent avec les données fournies : défaut du crate. La cause
    /// reste opaque pour ne pas lier l'API publique au moteur de templates.
    #[error("template rendering failed: {0}")]
    Rendering(#[source] Box<dyn StdError + Send + Sync>),
}
