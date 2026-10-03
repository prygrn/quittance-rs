use french_amount_words::{MAX_VALUE, euro_amount_to_words};
use quittance_core::{Receipt, ReceiptInput};
use time::macros::date;

use crate::test_support::{
    CHARGES_CENTS, LANDLORD_ADDRESS, LANDLORD_NAME, PROPERTY_ADDRESS, RENT_CENTS,
    SIGNATURE_DATA_URI, STANDARD_TEMPLATE_ID, TENANT_ADDRESS, TENANT_EMAIL, TENANT_NAME,
    receipt_from, receipt_with_amounts, render_standard, sample_input, sample_receipt,
};
use crate::{TemplateError, TemplateInfo, list_templates, render_html};

type TemplateListing = fn() -> Vec<TemplateInfo>;
type HtmlRendering = fn(&str, &Receipt, Option<&str>) -> Result<String, TemplateError>;

const STANDARD_TEMPLATE_LABEL: &str = "Quittance standard";
const CENTS_PER_EURO: u64 = 100;
/// Formats attendus avec l'espace fine insécable (U+202F) entre milliers
/// et l'espace insécable (U+00A0) avant le symbole euro.
const FORMATTED_RENT: &str = "650,00\u{a0}€";
const FORMATTED_CHARGES: &str = "50,50\u{a0}€";
const FORMATTED_TOTAL: &str = "700,50\u{a0}€";

fn assert_is_std_error<ErrorType: std::error::Error + Send + Sync + 'static>() {}

#[test]
fn given_crate_when_inspecting_public_api_then_exposes_listing_rendering_and_error() {
    // Arrange
    let listing: TemplateListing = list_templates;
    let rendering: HtmlRendering = render_html;

    // Act
    assert_is_std_error::<TemplateError>();

    // Assert
    let _ = (listing, rendering);
}

#[test]
fn given_catalog_when_listing_templates_then_only_standard_template_is_listed() {
    // Arrange
    let expected_templates = vec![TemplateInfo {
        id: STANDARD_TEMPLATE_ID,
        label: STANDARD_TEMPLATE_LABEL,
    }];

    // Act
    let templates = list_templates();

    // Assert
    assert_eq!(templates, expected_templates);
}

#[test]
fn given_catalog_when_listing_templates_twice_then_both_listings_are_identical() {
    // Arrange
    let first_listing = list_templates();

    // Act
    let second_listing = list_templates();

    // Assert
    assert_eq!(first_listing, second_listing);
}

#[test]
fn given_each_listed_template_when_rendering_then_rendering_succeeds() {
    // Arrange
    let receipt = sample_receipt();

    // Act
    let results: Vec<_> = list_templates()
        .into_iter()
        .map(|template| render_html(template.id, &receipt, None))
        .collect();

    // Assert
    assert!(results.iter().all(Result::is_ok), "{results:?}");
}

#[test]
fn given_receipt_when_rendering_standard_then_parties_and_property_appear() {
    // Arrange
    let receipt = sample_receipt();

    // Act
    let html = render_standard(&receipt);

    // Assert
    for expected in [
        LANDLORD_NAME,
        LANDLORD_ADDRESS,
        TENANT_NAME,
        TENANT_ADDRESS,
        PROPERTY_ADDRESS,
    ] {
        assert!(html.contains(expected), "missing `{expected}`");
    }
}

#[test]
fn given_receipt_when_rendering_standard_then_tenant_email_is_visible() {
    // Arrange
    let receipt = sample_receipt();

    // Act
    let html = render_standard(&receipt);

    // Assert
    assert!(html.contains(TENANT_EMAIL));
}

#[test]
fn given_receipt_when_rendering_standard_then_period_and_payment_dates_appear() {
    // Arrange
    let receipt = sample_receipt();

    // Act
    let html = render_standard(&receipt);

    // Assert
    for expected in ["01/10/2026", "31/10/2026", "05/10/2026"] {
        assert!(html.contains(expected), "missing `{expected}`");
    }
}

#[test]
fn given_receipt_when_rendering_standard_then_rent_charges_and_total_appear() {
    // Arrange
    let receipt = sample_receipt();

    // Act
    let html = render_standard(&receipt);

    // Assert
    for expected in [FORMATTED_RENT, FORMATTED_CHARGES, FORMATTED_TOTAL] {
        assert!(html.contains(expected), "missing `{expected}`");
    }
}

#[test]
fn given_receipt_when_rendering_standard_then_total_in_words_appears() {
    // Arrange
    let receipt = sample_receipt();
    let total_in_words = euro_amount_to_words(RENT_CENTS + CHARGES_CENTS).unwrap();

    // Act
    let html = render_standard(&receipt);

    // Assert
    assert!(html.contains(&total_in_words), "missing `{total_in_words}`");
}

#[test]
fn given_receipt_when_rendering_standard_then_receipt_formula_appears() {
    // Arrange
    let receipt = sample_receipt();

    // Act
    let html = render_standard(&receipt);

    // Assert
    for expected in [
        "déclare avoir reçu",
        "au titre du loyer et des charges",
        "en donne quittance, sous réserve de tous ses droits",
    ] {
        assert!(html.contains(expected), "missing `{expected}`");
    }
}

#[test]
fn given_markup_in_user_input_when_rendering_then_it_is_escaped() {
    // Arrange
    let receipt = receipt_from(ReceiptInput {
        tenant_name: "<script>alert(1)</script>".to_owned(),
        property_address: "<b>12 rue des Lilas</b>".to_owned(),
        ..sample_input()
    });

    // Act
    let html = render_standard(&receipt);

    // Assert
    assert!(!html.contains("<script>"));
    assert!(!html.contains("<b>"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&lt;b&gt;"));
}

#[test]
fn given_unknown_template_id_when_rendering_then_template_is_unknown() {
    // Arrange
    let receipt = sample_receipt();

    // Act
    let result = render_html("inconnu", &receipt, None);

    // Assert
    assert!(matches!(result, Err(TemplateError::UnknownTemplate(_))));
}

#[test]
fn given_signature_data_uri_when_rendering_then_signature_image_is_embedded() {
    // Arrange
    let receipt = sample_receipt();

    // Act
    let html = render_html(STANDARD_TEMPLATE_ID, &receipt, Some(SIGNATURE_DATA_URI)).unwrap();

    // Assert
    assert_eq!(html.matches("<img").count(), 1);
    assert!(html.contains("src=\"data:image"));
}

#[test]
fn given_no_signature_when_rendering_then_no_image_is_rendered() {
    // Arrange
    let receipt = sample_receipt();

    // Act
    let html = render_standard(&receipt);

    // Assert
    assert!(!html.contains("<img"));
}

#[test]
fn given_signature_pointing_to_remote_url_when_rendering_then_signature_is_invalid() {
    // Arrange
    let receipt = sample_receipt();

    // Act
    let result = render_html(
        STANDARD_TEMPLATE_ID,
        &receipt,
        Some("https://example.fr/signature.png"),
    );

    // Assert
    assert!(matches!(result, Err(TemplateError::InvalidSignature)));
}

#[test]
fn given_zero_amounts_when_rendering_then_amounts_read_zero_euros() {
    // Arrange
    let receipt = receipt_with_amounts(0, 0);

    // Act
    let html = render_standard(&receipt);

    // Assert
    assert!(html.contains("0,00\u{a0}€"));
}

#[test]
fn given_amount_below_one_euro_when_rendering_then_cents_are_zero_padded() {
    // Arrange
    let receipt = receipt_with_amounts(5, 0);

    // Act
    let html = render_standard(&receipt);

    // Assert
    assert!(html.contains("0,05\u{a0}€"));
}

#[test]
fn given_amount_above_one_thousand_euros_when_rendering_then_thousands_are_grouped() {
    // Arrange
    let receipt = receipt_with_amounts(123_456, 0);

    // Act
    let html = render_standard(&receipt);

    // Assert
    assert!(html.contains("1\u{202f}234,56\u{a0}€"));
}

#[test]
fn given_amount_above_one_million_euros_when_rendering_then_every_group_is_separated() {
    // Arrange
    let receipt = receipt_with_amounts(123_456_789, 0);

    // Act
    let html = render_standard(&receipt);

    // Assert
    assert!(html.contains("1\u{202f}234\u{202f}567,89\u{a0}€"));
}

#[test]
fn given_round_thousand_when_rendering_then_trailing_groups_keep_their_zeros() {
    // Arrange
    let receipt = receipt_with_amounts(100_000, 0);

    // Act
    let html = render_standard(&receipt);

    // Assert
    assert!(html.contains("1\u{202f}000,00\u{a0}€"));
}

#[test]
fn given_single_digit_days_and_months_when_rendering_then_dates_are_zero_padded() {
    // Arrange
    let receipt = receipt_from(ReceiptInput {
        period_start: date!(2026 - 02 - 01),
        period_end: date!(2026 - 02 - 09),
        payment_date: date!(2026 - 03 - 05),
        ..sample_input()
    });

    // Act
    let html = render_standard(&receipt);

    // Assert
    for expected in ["01/02/2026", "09/02/2026", "05/03/2026"] {
        assert!(html.contains(expected), "missing `{expected}`");
    }
}

#[test]
fn given_total_beyond_words_limit_when_rendering_then_amount_in_words_fails() {
    // Arrange
    let receipt = receipt_with_amounts((MAX_VALUE + 1) * CENTS_PER_EURO, 0);

    // Act
    let result = render_html(STANDARD_TEMPLATE_ID, &receipt, None);

    // Assert
    assert!(matches!(result, Err(TemplateError::AmountInWords(_))));
}

#[test]
fn given_signed_receipt_when_rendering_then_html_loads_no_script_nor_external_resource() {
    // Arrange
    let receipt = sample_receipt();

    // Act
    let html = render_html(STANDARD_TEMPLATE_ID, &receipt, Some(SIGNATURE_DATA_URI)).unwrap();

    // Assert
    for forbidden in [
        "<script", "<link", "href=", "url(", "@import", "http:", "https:",
    ] {
        assert!(!html.contains(forbidden), "found `{forbidden}`");
    }
    assert_eq!(
        html.matches("src=").count(),
        html.matches("src=\"data:").count()
    );
}

#[test]
fn given_receipt_when_rendering_then_document_is_a_standalone_a4_page() {
    // Arrange
    let receipt = sample_receipt();

    // Act
    let html = render_standard(&receipt);

    // Assert
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<meta charset=\"utf-8\">"));
    assert!(html.contains("<style>"));
    assert!(html.contains("size: A4"));
}
