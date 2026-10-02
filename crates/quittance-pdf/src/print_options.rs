use headless_chrome::types::PrintToPdfOptions;

/// Mise en page d'une quittance ; le protocole CDP attend des pouces.
struct ReceiptPageLayout;

impl ReceiptPageLayout {
    const MILLIMETERS_PER_INCH: f64 = 25.4;
    const A4_WIDTH_MILLIMETERS: f64 = 210.0;
    const A4_HEIGHT_MILLIMETERS: f64 = 297.0;
    const MARGIN_MILLIMETERS: f64 = 10.0;

    fn to_inches(millimeters: f64) -> f64 {
        millimeters / Self::MILLIMETERS_PER_INCH
    }
}

/// Options d'impression d'une quittance : A4, marges uniformes, fond imprimé.
pub(crate) fn build_print_options() -> PrintToPdfOptions {
    let margin = Some(ReceiptPageLayout::to_inches(
        ReceiptPageLayout::MARGIN_MILLIMETERS,
    ));
    PrintToPdfOptions {
        print_background: Some(true),
        paper_width: Some(ReceiptPageLayout::to_inches(
            ReceiptPageLayout::A4_WIDTH_MILLIMETERS,
        )),
        paper_height: Some(ReceiptPageLayout::to_inches(
            ReceiptPageLayout::A4_HEIGHT_MILLIMETERS,
        )),
        margin_top: margin,
        margin_bottom: margin,
        margin_left: margin,
        margin_right: margin,
        ..PrintToPdfOptions::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MILLIMETERS_PER_INCH: f64 = 25.4;
    const TOLERANCE_MILLIMETERS: f64 = 0.01;

    fn to_millimeters(inches: Option<f64>) -> f64 {
        inches.expect("dimension must be set") * MILLIMETERS_PER_INCH
    }

    fn assert_millimeters(inches: Option<f64>, expected_millimeters: f64) {
        let actual_millimeters = to_millimeters(inches);
        assert!(
            (actual_millimeters - expected_millimeters).abs() < TOLERANCE_MILLIMETERS,
            "expected {expected_millimeters} mm, got {actual_millimeters} mm"
        );
    }

    #[test]
    fn given_receipt_layout_when_building_print_options_then_paper_is_a4() {
        let options = build_print_options();

        assert_millimeters(options.paper_width, 210.0);
        assert_millimeters(options.paper_height, 297.0);
    }

    #[test]
    fn given_receipt_layout_when_building_print_options_then_all_margins_are_ten_millimeters() {
        let options = build_print_options();

        for margin in [
            options.margin_top,
            options.margin_bottom,
            options.margin_left,
            options.margin_right,
        ] {
            assert_millimeters(margin, 10.0);
        }
    }

    #[test]
    fn given_receipt_layout_when_building_print_options_then_background_is_printed() {
        let options = build_print_options();

        assert_eq!(options.print_background, Some(true));
    }
}
