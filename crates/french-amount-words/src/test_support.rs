//! Outillage partagé par les tests table-driven du crate.

use crate::AmountWordsError;

pub(crate) type Conversion = fn(u64) -> Result<String, AmountWordsError>;

/// Vérifie chaque couple (entrée, sortie attendue) en signalant l'entrée fautive.
pub(crate) fn assert_conversions(convert: Conversion, cases: &[(u64, &str)]) {
    for &(input, expected) in cases {
        let actual = convert(input);

        assert_eq!(actual.as_deref(), Ok(expected), "input: {input}");
    }
}
