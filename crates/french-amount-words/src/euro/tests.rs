use crate::test_support::assert_conversions;
use crate::{AmountWordsError, MAX_VALUE, euro_amount_to_words};

const CENTS_PER_EURO: u64 = 100;

#[test]
fn given_zero_cents_when_converted_then_returns_zero_euro_in_singular() {
    // Arrange
    let cases = [(0, "zéro euro")];

    // Act & Assert
    assert_conversions(euro_amount_to_words, &cases);
}

#[test]
fn given_one_euro_when_converted_then_euro_stays_singular() {
    // Arrange
    let cases = [(100, "un euro")];

    // Act & Assert
    assert_conversions(euro_amount_to_words, &cases);
}

#[test]
fn given_whole_euros_when_converted_then_omits_zero_centime() {
    // Arrange
    let cases = [
        (200, "deux euros"),
        (2_100, "vingt et un euros"),
        (8_000, "quatre-vingts euros"),
        (100_000, "mille euros"),
    ];

    // Act & Assert
    assert_conversions(euro_amount_to_words, &cases);
}

#[test]
fn given_cents_only_when_converted_then_omits_zero_euro() {
    // Arrange
    let cases = [
        (1, "un centime"),
        (50, "cinquante centimes"),
        (99, "quatre-vingt-dix-neuf centimes"),
    ];

    // Act & Assert
    assert_conversions(euro_amount_to_words, &cases);
}

#[test]
fn given_euros_and_cents_when_converted_then_joins_them_with_et() {
    // Arrange
    let cases = [
        (101, "un euro et un centime"),
        (250, "deux euros et cinquante centimes"),
        (
            123_456,
            "mille deux cent trente-quatre euros et cinquante-six centimes",
        ),
    ];

    // Act & Assert
    assert_conversions(euro_amount_to_words, &cases);
}

#[test]
fn given_euros_ending_with_million_or_milliard_when_converted_then_elides_de() {
    // Arrange
    let cases = [
        (100_000_000, "un million d'euros"),
        (200_000_000, "deux millions d'euros"),
        (100_000_050, "un million d'euros et cinquante centimes"),
        (100_000_000_000, "un milliard d'euros"),
        (150_000_000_000, "un milliard cinq cents millions d'euros"),
    ];

    // Act & Assert
    assert_conversions(euro_amount_to_words, &cases);
}

#[test]
fn given_million_followed_by_number_when_converted_then_uses_euros_without_de() {
    // Arrange
    let cases = [
        (100_020_000, "un million deux cents euros"),
        (100_100_000, "un million mille euros"),
    ];

    // Act & Assert
    assert_conversions(euro_amount_to_words, &cases);
}

#[test]
fn given_max_euro_part_when_converted_then_is_accepted() {
    // Arrange
    let largest_amount = MAX_VALUE * CENTS_PER_EURO + (CENTS_PER_EURO - 1);

    // Act
    let result = euro_amount_to_words(largest_amount);

    // Assert
    assert!(result.is_ok(), "input: {largest_amount}");
}

#[test]
fn given_euro_part_above_max_value_when_converted_then_returns_value_too_large_error() {
    // Arrange
    let too_large_amounts = [(MAX_VALUE + 1) * CENTS_PER_EURO, u64::MAX];

    for amount in too_large_amounts {
        // Act
        let result = euro_amount_to_words(amount);

        // Assert
        assert!(
            matches!(result, Err(AmountWordsError::ValueTooLarge { .. })),
            "input: {amount}"
        );
    }
}
