use french_amount_words::AmountWordsError;
use thiserror::Error;

/// Échec du rendu HTML d'une quittance.
#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("template `{0}` does not exist")]
    UnknownTemplate(String),
    /// Seule une image embarquée est acceptée : le PDF est imprimé sans accès réseau.
    #[error("signature is not an embedded image data URI")]
    InvalidSignature,
    /// Montant total au-delà de [`french_amount_words::MAX_VALUE`] euros.
    #[error("total amount cannot be written in words: {0}")]
    AmountInWords(#[from] AmountWordsError),
    /// Template embarqué incohérent avec les données fournies : défaut du crate.
    #[error("template rendering failed: {0}")]
    Rendering(#[from] minijinja::Error),
}
