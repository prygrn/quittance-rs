use std::fmt;

use crate::MAX_VALUE;

/// Échec de conversion d'un nombre ou d'un montant en lettres.
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
