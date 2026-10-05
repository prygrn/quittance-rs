use std::fmt;

use crate::MAX_VALUE;

/// Échec de conversion d'un nombre ou d'un montant en lettres.
///
/// Un `match` hors du crate doit prévoir un bras `_`, afin que de nouvelles variantes puissent
/// être ajoutées sans rupture de compatibilité :
///
/// ```compile_fail,E0004
/// use french_amount_words::AmountWordsError;
///
/// fn too_large_value(error: &AmountWordsError) -> u64 {
///     match error {
///         AmountWordsError::ValueTooLarge { value } => *value,
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmountWordsError {
    /// La valeur dépasse [`crate::MAX_VALUE`].
    ValueTooLarge {
        /// Valeur comparée à [`crate::MAX_VALUE`] : l'entrée de [`crate::number_to_words`],
        /// mais la partie en euros (et non l'entrée en centimes) pour
        /// [`crate::euro_amount_to_words`].
        value: u64,
    },
}

impl fmt::Display for AmountWordsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValueTooLarge { value } => write!(
                formatter,
                "value {value} exceeds the maximum convertible value {MAX_VALUE}"
            ),
        }
    }
}

impl std::error::Error for AmountWordsError {}
