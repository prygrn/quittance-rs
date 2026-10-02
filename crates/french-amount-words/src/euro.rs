use crate::AmountWordsError;
use crate::number::{MILLION, number_to_words};

const CENTS_PER_EURO: u64 = 100;
/// Zéro et un gouvernent le singulier : zéro euro, un euro, un centime.
const LARGEST_SINGULAR_QUANTITY: u64 = 1;

const EURO_SINGULAR: &str = "euro";
const EURO_PLURAL: &str = "euros";
/// Après « million » ou « milliard », noms, le complément se construit avec « de ».
const EURO_AFTER_SCALE_NOUN: &str = "d'euros";
const CENTIME_SINGULAR: &str = "centime";
const CENTIME_PLURAL: &str = "centimes";

/// Écrit un montant exprimé en centimes d'euro en toutes lettres.
///
/// # Errors
///
/// Renvoie [`AmountWordsError::ValueTooLarge`] si la partie en euros dépasse
/// [`crate::MAX_VALUE`].
pub fn euro_amount_to_words(cents: u64) -> Result<String, AmountWordsError> {
    let euros = cents / CENTS_PER_EURO;
    let remaining_cents = cents % CENTS_PER_EURO;
    let euro_words = format!("{} {}", number_to_words(euros)?, euro_unit(euros));

    if remaining_cents == 0 {
        return Ok(euro_words);
    }

    let centime_words = format!(
        "{} {}",
        number_to_words(remaining_cents)?,
        centime_unit(remaining_cents)
    );
    if euros == 0 {
        return Ok(centime_words);
    }

    Ok(format!("{euro_words} et {centime_words}"))
}

fn euro_unit(euros: u64) -> &'static str {
    if euros <= LARGEST_SINGULAR_QUANTITY {
        EURO_SINGULAR
    } else if euros % MILLION == 0 {
        EURO_AFTER_SCALE_NOUN
    } else {
        EURO_PLURAL
    }
}

fn centime_unit(remaining_cents: u64) -> &'static str {
    if remaining_cents <= LARGEST_SINGULAR_QUANTITY {
        CENTIME_SINGULAR
    } else {
        CENTIME_PLURAL
    }
}

#[cfg(test)]
mod tests;
