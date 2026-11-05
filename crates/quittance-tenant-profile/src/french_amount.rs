//! Montants en euros à la française, tels qu'écrits dans une fiche : `650,00`.

use crate::TenantProfileError;

mod amount_syntax {
    pub const CENTS_PER_EURO: u64 = 100;
    pub const DECIMAL_SEPARATOR: char = ',';
    pub const MAXIMUM_DECIMAL_DIGITS: usize = 2;
}

/// Montant écrit en euros, avec une virgule et toujours deux décimales : `650,00`.
pub(crate) fn formatted_amount(cents: u64) -> String {
    format!(
        "{}{}{:02}",
        cents / amount_syntax::CENTS_PER_EURO,
        amount_syntax::DECIMAL_SEPARATOR,
        cents % amount_syntax::CENTS_PER_EURO
    )
}

/// Montant en centimes d'un texte au format `\d+(,\d{1,2})?` : `650`, `650,5`, `650,50`.
pub(crate) fn parsed_amount(field: &'static str, value: &str) -> Result<u64, TenantProfileError> {
    cents_of(value).ok_or_else(|| TenantProfileError::InvalidAmount {
        field,
        value: value.to_owned(),
    })
}

fn cents_of(value: &str) -> Option<u64> {
    let (euros, decimals) = value
        .split_once(amount_syntax::DECIMAL_SEPARATOR)
        .unwrap_or((value, "0"));
    if !is_digit_run(euros)
        || !is_digit_run(decimals)
        || decimals.len() > amount_syntax::MAXIMUM_DECIMAL_DIGITS
    {
        return None;
    }
    let decimal_cents = match decimals.len() {
        1 => decimals.parse::<u64>().ok()? * 10,
        _ => decimals.parse::<u64>().ok()?,
    };
    euros
        .parse::<u64>()
        .ok()?
        .checked_mul(amount_syntax::CENTS_PER_EURO)?
        .checked_add(decimal_cents)
}

fn is_digit_run(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|character| character.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_cents_when_formatting_then_euros_and_two_decimals_are_written_without_grouping() {
        let expectations = [
            (65_000, "650,00"),
            (5_050, "50,50"),
            (5, "0,05"),
            (0, "0,00"),
            (123_456_789, "1234567,89"),
        ];

        for (cents, expected) in expectations {
            assert_eq!(formatted_amount(cents), expected, "{cents} cents");
        }
    }

    #[test]
    fn given_french_amount_when_parsing_then_cents_are_returned() {
        let expectations = [
            ("650,00", 65_000),
            ("50,5", 5_050),
            ("50,05", 5_005),
            ("650", 65_000),
            ("0,05", 5),
            ("007,10", 710),
        ];

        for (value, expected) in expectations {
            assert_eq!(parsed_amount("rent", value), Ok(expected), "{value:?}");
        }
    }

    #[test]
    fn given_formatted_amount_when_parsing_then_original_cents_are_returned() {
        for cents in [0, 7, 65_000, u64::MAX] {
            assert_eq!(parsed_amount("rent", &formatted_amount(cents)), Ok(cents));
        }
    }

    #[test]
    fn given_malformed_amount_when_parsing_then_field_and_value_are_reported() {
        for value in [
            "",
            "650.00",
            "650,",
            ",50",
            "650,005",
            "-5",
            "+5",
            "6 500,00",
            "650,00 €",
            "1e3",
            "٣",
            "50,5a",
        ] {
            let result = parsed_amount("charges", value);

            assert_eq!(
                result,
                Err(TenantProfileError::InvalidAmount {
                    field: "charges",
                    value: value.to_owned()
                }),
                "{value:?}"
            );
        }
    }

    #[test]
    fn given_amount_beyond_the_cents_range_when_parsing_then_it_is_rejected() {
        for value in ["184467440737095516,16", "99999999999999999999"] {
            let result = parsed_amount("rent", value);

            assert!(
                matches!(result, Err(TenantProfileError::InvalidAmount { .. })),
                "{value:?} got {result:?}"
            );
        }
    }
}
