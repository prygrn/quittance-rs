use headless_chrome::types::PrintToPdfOptions;

/// Options d'impression d'une quittance : A4, marges uniformes, fond imprimé.
pub(crate) fn build_print_options() -> PrintToPdfOptions {
    todo!()
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
