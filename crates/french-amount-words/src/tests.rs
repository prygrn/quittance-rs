use crate::test_support::Conversion;
use crate::{AmountWordsError, MAX_VALUE, euro_amount_to_words, number_to_words};

const EXPECTED_MAX_VALUE: u64 = 999_999_999_999;

#[test]
fn given_crate_when_inspecting_public_api_then_exposes_conversions_bound_and_error() {
    // Arrange
    let number_conversion: Conversion = number_to_words;
    let euro_conversion: Conversion = euro_amount_to_words;

    // Act
    let error_as_std_error: &dyn std::error::Error =
        &AmountWordsError::ValueTooLarge { value: MAX_VALUE };

    // Assert
    assert_eq!(MAX_VALUE, EXPECTED_MAX_VALUE);
    assert!(error_as_std_error.source().is_none());
    let _ = (number_conversion, euro_conversion);
}

#[test]
fn given_value_too_large_error_when_displayed_then_message_is_not_empty() {
    // Arrange
    let error = AmountWordsError::ValueTooLarge {
        value: MAX_VALUE + 1,
    };

    // Act
    let message = error.to_string();

    // Assert
    assert!(!message.is_empty());
}
