use crate::AmountWordsError;

/// Écrit un montant exprimé en centimes d'euro en toutes lettres.
///
/// # Errors
///
/// Renvoie [`AmountWordsError::ValueTooLarge`] si la partie en euros dépasse
/// [`crate::MAX_VALUE`].
pub fn euro_amount_to_words(cents: u64) -> Result<String, AmountWordsError> {
    todo!()
}

#[cfg(test)]
mod tests;
