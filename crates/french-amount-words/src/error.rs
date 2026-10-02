use std::fmt;

/// Échec de conversion d'un nombre ou d'un montant en lettres.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmountWordsError {
    /// La valeur dépasse [`crate::MAX_VALUE`].
    ValueTooLarge { value: u64 },
}

impl fmt::Display for AmountWordsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for AmountWordsError {}
