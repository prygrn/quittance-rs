use quittance_core::{Date, Money};

/// Typographie française des nombres, alignée sur les données CLDR de la locale `fr`.
mod typography {
    pub const CENTS_PER_EURO: u64 = 100;
    pub const DIGIT_GROUP_SIZE: usize = 3;
    /// Espace fine insécable (U+202F) : sépare les milliers sans couper le nombre en fin de ligne.
    pub const THOUSANDS_SEPARATOR: char = '\u{202F}';
    pub const DECIMAL_SEPARATOR: char = ',';
    /// Espace insécable (U+00A0) : le symbole ne se retrouve jamais seul en début de ligne.
    pub const CURRENCY_SEPARATOR: char = '\u{00A0}';
    pub const CURRENCY_SYMBOL: char = '€';
}

/// Montant au format `1 234,56 €`.
pub(crate) fn formatted_money(money: Money) -> String {
    let euros = money.cents() / typography::CENTS_PER_EURO;
    let cents = money.cents() % typography::CENTS_PER_EURO;
    format!(
        "{}{}{cents:02}{}{}",
        grouped_digits(euros),
        typography::DECIMAL_SEPARATOR,
        typography::CURRENCY_SEPARATOR,
        typography::CURRENCY_SYMBOL,
    )
}

/// Date au format `JJ/MM/AAAA`.
pub(crate) fn formatted_date(date: Date) -> String {
    format!(
        "{:02}/{:02}/{:04}",
        date.day(),
        u8::from(date.month()),
        date.year()
    )
}

fn grouped_digits(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        let remaining_digit_count = digits.len() - index;
        if index > 0 && remaining_digit_count.is_multiple_of(typography::DIGIT_GROUP_SIZE) {
            grouped.push(typography::THOUSANDS_SEPARATOR);
        }
        grouped.push(digit);
    }
    grouped
}
