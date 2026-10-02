use crate::{AmountWordsError, MAX_VALUE};

pub(crate) const MILLION: u64 = 1_000_000;

const MILLIARD: u64 = 1_000_000_000;
const THOUSAND: u64 = 1_000;
const HUNDRED: u64 = 100;
const SMALLEST_PLURAL_QUANTITY: u64 = 2;

const ZERO_WORD: &str = "zéro";
const HUNDRED_WORD: &str = "cent";
const THOUSAND_WORD: &str = "mille";
const PLURAL_MARK: &str = "s";
const WORD_SEPARATOR: &str = " ";

/// Mots de 0 à 19, indexés par leur valeur ; zéro n'apparaît jamais dans un nombre composé.
const BELOW_TWENTY_WORDS: [&str; 20] = [
    "", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix", "onze",
    "douze", "treize", "quatorze", "quinze", "seize", "dix-sept", "dix-huit", "dix-neuf",
];

/// Restes joints par « et » après une dizaine : vingt et un, soixante et onze.
const REMAINDERS_JOINED_WITH_ET: [u64; 2] = [1, 11];

/// « quatre-vingt » multiplie « vingt » : il prend le pluriel et refuse « et ».
const MULTIPLIED_VINGT: u64 = 80;

/// Dizaines de base par valeur croissante ; 70 et 90 se forment sur soixante et
/// quatre-vingt suivis de 10 à 19, d'où l'absence de bases à 70 et 90.
const TENS: [(u64, &str); 6] = [
    (20, "vingt"),
    (30, "trente"),
    (40, "quarante"),
    (50, "cinquante"),
    (60, "soixante"),
    (MULTIPLIED_VINGT, "quatre-vingt"),
];

/// Noms d'échelle : contrairement à « mille », ils s'accordent et laissent vingt et cent
/// s'accorder devant eux.
const SCALE_NOUNS: [(u64, &str); 2] = [(MILLIARD, "milliard"), (MILLION, "million")];

/// Nombre grammatical d'un mot, qui décide de sa marque du pluriel.
#[derive(Clone, Copy)]
enum GrammaticalNumber {
    Singular,
    Plural,
}

impl GrammaticalNumber {
    fn of_quantity(quantity: u64) -> Self {
        if quantity >= SMALLEST_PLURAL_QUANTITY {
            Self::Plural
        } else {
            Self::Singular
        }
    }

    fn plural_mark(self) -> &'static str {
        match self {
            Self::Singular => "",
            Self::Plural => PLURAL_MARK,
        }
    }
}

/// Écrit un nombre entier en toutes lettres, en graphie traditionnelle.
///
/// # Errors
///
/// Renvoie [`AmountWordsError::ValueTooLarge`] si `value` dépasse [`crate::MAX_VALUE`].
pub fn number_to_words(value: u64) -> Result<String, AmountWordsError> {
    if value > MAX_VALUE {
        return Err(AmountWordsError::ValueTooLarge { value });
    }
    if value == 0 {
        return Ok(ZERO_WORD.to_owned());
    }

    let mut parts: Vec<String> = Vec::new();
    for (scale, noun) in SCALE_NOUNS {
        let count = value / scale % THOUSAND;
        if count > 0 {
            parts.push(format!(
                "{} {noun}{}",
                group_words(count, GrammaticalNumber::Plural),
                GrammaticalNumber::of_quantity(count).plural_mark()
            ));
        }
    }

    let thousands = value / THOUSAND % THOUSAND;
    match thousands {
        0 => {}
        1 => parts.push(THOUSAND_WORD.to_owned()),
        // Devant « mille », adjectif numéral, vingt et cent restent invariables :
        // quatre-vingt mille, deux cent mille.
        _ => parts.push(format!(
            "{} {THOUSAND_WORD}",
            group_words(thousands, GrammaticalNumber::Singular)
        )),
    }

    let units = value % THOUSAND;
    if units > 0 {
        parts.push(group_words(units, GrammaticalNumber::Plural));
    }

    Ok(parts.join(WORD_SEPARATOR))
}

/// Écrit un groupe de 1 à 999 ; `ending_number` est le nombre que prennent « vingt » et
/// « cent » multipliés quand ils terminent le groupe.
fn group_words(value: u64, ending_number: GrammaticalNumber) -> String {
    let hundreds = value / HUNDRED;
    let below_hundred = value % HUNDRED;

    let mut parts: Vec<String> = Vec::new();
    match hundreds {
        0 => {}
        1 => parts.push(HUNDRED_WORD.to_owned()),
        _ => {
            let hundred_number = if below_hundred == 0 {
                ending_number
            } else {
                GrammaticalNumber::Singular
            };
            parts.push(format!(
                "{} {HUNDRED_WORD}{}",
                below_twenty_word(hundreds),
                hundred_number.plural_mark()
            ));
        }
    }
    if below_hundred > 0 {
        parts.push(below_hundred_words(below_hundred, ending_number));
    }

    parts.join(WORD_SEPARATOR)
}

/// Écrit un nombre de 1 à 99, en liant dizaines et unités par un trait d'union ou par « et ».
fn below_hundred_words(value: u64, ending_number: GrammaticalNumber) -> String {
    let Some(&(tens_value, tens_word)) = TENS
        .iter()
        .rev()
        .find(|(tens_value, _)| *tens_value <= value)
    else {
        return below_twenty_word(value).to_owned();
    };

    let is_multiplied_vingt = tens_value == MULTIPLIED_VINGT;
    let remainder = value - tens_value;
    if remainder == 0 {
        let tens_number = if is_multiplied_vingt {
            ending_number
        } else {
            GrammaticalNumber::Singular
        };
        return format!("{tens_word}{}", tens_number.plural_mark());
    }

    let remainder_word = below_twenty_word(remainder);
    if !is_multiplied_vingt && REMAINDERS_JOINED_WITH_ET.contains(&remainder) {
        format!("{tens_word} et {remainder_word}")
    } else {
        format!("{tens_word}-{remainder_word}")
    }
}

fn below_twenty_word(value: u64) -> &'static str {
    BELOW_TWENTY_WORDS[value as usize]
}

#[cfg(test)]
mod tests;
