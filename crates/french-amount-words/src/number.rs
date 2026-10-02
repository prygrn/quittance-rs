use crate::AmountWordsError;

/// Écrit un nombre entier en toutes lettres, en graphie traditionnelle.
///
/// # Errors
///
/// Renvoie [`AmountWordsError::ValueTooLarge`] si `value` dépasse [`crate::MAX_VALUE`].
pub fn number_to_words(value: u64) -> Result<String, AmountWordsError> {
    todo!()
}

#[cfg(test)]
mod tests;
