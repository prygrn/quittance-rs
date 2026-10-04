use quittance_core::Date;

/// Lit une date au format ISO `YYYY-MM-DD`, celui d'un `<input type="date">`.
/// Renvoie `None` pour toute autre forme ou pour une date inexistante.
pub fn parse_iso_date(text: &str) -> Option<Date> {
    let bytes = text.as_bytes();
    let is_iso_shape = bytes.len() == iso_layout::LENGTH
        && iso_layout::SEPARATOR_POSITIONS
            .iter()
            .all(|&position| bytes[position] == iso_layout::SEPARATOR)
        && bytes
            .iter()
            .enumerate()
            .filter(|(position, _)| !iso_layout::SEPARATOR_POSITIONS.contains(position))
            .all(|(_, byte)| byte.is_ascii_digit());
    if !is_iso_shape {
        return None;
    }
    // Forme vérifiée : uniquement des chiffres ASCII aux positions découpées.
    let year: i32 = text[0..4].parse().ok()?;
    let month: u8 = text[5..7].parse().ok()?;
    let day: u8 = text[8..10].parse().ok()?;
    Date::from_calendar_date(year, month.try_into().ok()?, day).ok()
}

/// Forme `YYYY-MM-DD` : dix caractères, tirets en cinquième et huitième positions.
mod iso_layout {
    pub const LENGTH: usize = 10;
    pub const SEPARATOR: u8 = b'-';
    pub const SEPARATOR_POSITIONS: [usize; 2] = [4, 7];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::date;

    #[test]
    fn given_iso_date_when_parsing_then_calendar_date_is_read() {
        let parsed = parse_iso_date("2026-10-05");

        assert_eq!(parsed, Some(date(2026, 10, 5)));
    }

    #[test]
    fn given_leap_day_when_parsing_then_it_is_accepted() {
        let parsed = parse_iso_date("2028-02-29");

        assert_eq!(parsed, Some(date(2028, 2, 29)));
    }

    #[test]
    fn given_dates_that_do_not_exist_when_parsing_then_each_is_rejected() {
        for text in [
            "2026-02-29",
            "2026-13-01",
            "2026-00-10",
            "2026-04-31",
            "2026-10-00",
        ] {
            assert_eq!(parse_iso_date(text), None, "{text} should be rejected");
        }
    }

    #[test]
    fn given_malformed_texts_when_parsing_then_each_is_rejected() {
        let malformed_texts = [
            "",
            "05/10/2026",
            "2026-10-5",
            "2026-1-05",
            "26-10-05",
            "2026/10/05",
            "2026-10-05T00:00",
            " 2026-10-05",
            "+2026-10-05",
            "2026-1a-05",
            "２０２６-10-05",
        ];

        for text in malformed_texts {
            assert_eq!(parse_iso_date(text), None, "{text:?} should be rejected");
        }
    }
}
