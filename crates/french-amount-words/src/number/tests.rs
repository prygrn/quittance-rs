use crate::test_support::assert_conversions;
use crate::{AmountWordsError, MAX_VALUE, number_to_words};

#[test]
fn given_zero_when_converted_then_returns_zero_word() {
    // Arrange
    let cases = [(0, "zéro")];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_units_and_teens_when_converted_then_returns_simple_words() {
    // Arrange
    let cases = [
        (1, "un"),
        (2, "deux"),
        (7, "sept"),
        (9, "neuf"),
        (10, "dix"),
        (11, "onze"),
        (12, "douze"),
        (16, "seize"),
    ];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_seventeen_to_nineteen_when_converted_then_hyphenates_ten_and_unit() {
    // Arrange
    let cases = [(17, "dix-sept"), (18, "dix-huit"), (19, "dix-neuf")];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_round_tens_when_converted_then_returns_tens_word() {
    // Arrange
    let cases = [
        (20, "vingt"),
        (30, "trente"),
        (40, "quarante"),
        (50, "cinquante"),
        (60, "soixante"),
        (70, "soixante-dix"),
        (90, "quatre-vingt-dix"),
    ];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_tens_ending_with_one_below_eighty_when_converted_then_joins_with_et_without_hyphen() {
    // Arrange
    let cases = [
        (21, "vingt et un"),
        (31, "trente et un"),
        (41, "quarante et un"),
        (51, "cinquante et un"),
        (61, "soixante et un"),
        (71, "soixante et onze"),
    ];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_compound_tens_without_et_when_converted_then_hyphenates_tens_and_units() {
    // Arrange
    let cases = [
        (22, "vingt-deux"),
        (35, "trente-cinq"),
        (48, "quarante-huit"),
        (59, "cinquante-neuf"),
        (66, "soixante-six"),
        (72, "soixante-douze"),
        (77, "soixante-dix-sept"),
        (79, "soixante-dix-neuf"),
    ];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_eighty_when_converted_then_vingt_takes_plural() {
    // Arrange
    let cases = [(80, "quatre-vingts")];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_eighties_and_nineties_when_converted_then_hyphenates_without_et_nor_plural() {
    // Arrange
    let cases = [
        (81, "quatre-vingt-un"),
        (82, "quatre-vingt-deux"),
        (89, "quatre-vingt-neuf"),
        (91, "quatre-vingt-onze"),
        (97, "quatre-vingt-dix-sept"),
        (99, "quatre-vingt-dix-neuf"),
    ];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_one_hundred_when_converted_then_returns_cent_without_un() {
    // Arrange
    let cases = [(100, "cent"), (101, "cent un"), (180, "cent quatre-vingts")];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_multiplied_round_hundreds_when_converted_then_cent_takes_plural() {
    // Arrange
    let cases = [
        (200, "deux cents"),
        (500, "cinq cents"),
        (900, "neuf cents"),
    ];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_hundreds_followed_by_number_when_converted_then_cent_stays_singular() {
    // Arrange
    let cases = [
        (201, "deux cent un"),
        (221, "deux cent vingt et un"),
        (280, "deux cent quatre-vingts"),
        (999, "neuf cent quatre-vingt-dix-neuf"),
    ];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_one_thousand_when_converted_then_returns_mille_without_un() {
    // Arrange
    let cases = [(1_000, "mille"), (1_001, "mille un"), (1_100, "mille cent")];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_multiplied_thousands_when_converted_then_mille_stays_invariable() {
    // Arrange
    let cases = [
        (1_234, "mille deux cent trente-quatre"),
        (2_000, "deux mille"),
        (21_000, "vingt et un mille"),
        (
            999_999,
            "neuf cent quatre-vingt-dix-neuf mille neuf cent quatre-vingt-dix-neuf",
        ),
    ];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_vingt_or_cent_before_mille_when_converted_then_they_stay_singular() {
    // Arrange
    let cases = [
        (80_000, "quatre-vingt mille"),
        (81_000, "quatre-vingt-un mille"),
        (200_000, "deux cent mille"),
        (280_000, "deux cent quatre-vingt mille"),
    ];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_millions_when_converted_then_million_is_a_noun_that_takes_plural() {
    // Arrange
    let cases = [
        (1_000_000, "un million"),
        (2_000_000, "deux millions"),
        (21_000_000, "vingt et un millions"),
        (1_000_200, "un million deux cents"),
        (1_080_000, "un million quatre-vingt mille"),
        (2_001_001, "deux millions mille un"),
    ];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_vingt_or_cent_before_million_when_converted_then_they_take_plural() {
    // Arrange
    let cases = [
        (80_000_000, "quatre-vingts millions"),
        (200_000_000, "deux cents millions"),
    ];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_milliards_when_converted_then_milliard_is_a_noun_that_takes_plural() {
    // Arrange
    let cases = [
        (1_000_000_000, "un milliard"),
        (2_000_000_000, "deux milliards"),
        (1_000_000_001, "un milliard un"),
        (1_500_000_000, "un milliard cinq cents millions"),
        (80_000_000_000, "quatre-vingts milliards"),
        (200_000_000_000, "deux cents milliards"),
    ];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_max_value_when_converted_then_returns_full_words() {
    // Arrange
    let cases = [(
        MAX_VALUE,
        "neuf cent quatre-vingt-dix-neuf milliards neuf cent quatre-vingt-dix-neuf millions \
         neuf cent quatre-vingt-dix-neuf mille neuf cent quatre-vingt-dix-neuf",
    )];

    // Act & Assert
    assert_conversions(number_to_words, &cases);
}

#[test]
fn given_values_above_max_value_when_converted_then_returns_value_too_large_error() {
    // Arrange
    let too_large_values = [MAX_VALUE + 1, u64::MAX];

    for value in too_large_values {
        // Act
        let result = number_to_words(value);

        // Assert
        assert!(
            matches!(result, Err(AmountWordsError::ValueTooLarge { .. })),
            "input: {value}"
        );
    }
}
