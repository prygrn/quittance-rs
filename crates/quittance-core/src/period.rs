use time::Date;

use crate::ReceiptError;

/// Période couverte par la quittance, bornes incluses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RentPeriod {
    start: Date,
    end: Date,
}

impl RentPeriod {
    pub(crate) fn new(start: Date, end: Date) -> Result<Self, ReceiptError> {
        if start > end {
            return Err(ReceiptError::InvertedPeriod { start, end });
        }
        Ok(Self { start, end })
    }

    pub fn start(&self) -> Date {
        self.start
    }

    pub fn end(&self) -> Date {
        self.end
    }
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    #[test]
    fn given_start_before_end_when_creating_period_then_it_exposes_both_bounds() {
        let period = RentPeriod::new(date!(2026 - 10 - 01), date!(2026 - 10 - 31)).unwrap();

        assert_eq!(period.start(), date!(2026 - 10 - 01));
        assert_eq!(period.end(), date!(2026 - 10 - 31));
    }

    #[test]
    fn given_same_start_and_end_when_creating_period_then_one_day_period_is_accepted() {
        let result = RentPeriod::new(date!(2026 - 10 - 15), date!(2026 - 10 - 15));

        assert!(result.is_ok());
    }

    #[test]
    fn given_start_after_end_when_creating_period_then_period_is_inverted() {
        let result = RentPeriod::new(date!(2026 - 11 - 01), date!(2026 - 10 - 31));

        assert!(matches!(result, Err(ReceiptError::InvertedPeriod { .. })));
    }
}
