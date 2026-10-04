use std::time::{SystemTime, UNIX_EPOCH};

use quittance_core::Date;

/// Horloge donnant la date de génération d'une quittance, derrière un trait pour que les
/// tests fixent la date.
pub trait Clock {
    fn today(&self) -> Date;
}

/// Horloge système. La date est celle du méridien de Greenwich : la date locale exigerait
/// un fuseau horaire, que la bibliothèque standard ne fournit pas.
pub struct SystemClock;

impl Clock for SystemClock {
    fn today(&self) -> Date {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is set before 1970-01-01");
        date_from_unix_seconds(elapsed.as_secs())
    }
}

/// Date UTC du nombre de secondes écoulées depuis le 1er janvier 1970.
fn date_from_unix_seconds(seconds: u64) -> Date {
    let _ = seconds;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::date;

    const SECONDS_PER_DAY: u64 = 86_400;
    /// 2026-10-04T00:00:00Z.
    const OCTOBER_4_2026_MIDNIGHT: u64 = 1_791_072_000;

    #[test]
    fn given_unix_epoch_when_converting_then_date_is_january_first_1970() {
        assert_eq!(date_from_unix_seconds(0), date(1970, 1, 1));
    }

    #[test]
    fn given_midnight_when_converting_then_date_is_that_day() {
        assert_eq!(
            date_from_unix_seconds(OCTOBER_4_2026_MIDNIGHT),
            date(2026, 10, 4)
        );
    }

    #[test]
    fn given_last_second_of_a_day_when_converting_then_date_is_still_that_day() {
        let last_second = OCTOBER_4_2026_MIDNIGHT + SECONDS_PER_DAY - 1;

        assert_eq!(date_from_unix_seconds(last_second), date(2026, 10, 4));
    }

    #[test]
    fn given_system_clock_when_reading_today_then_date_is_not_before_this_code_was_written() {
        assert!(SystemClock.today() >= date(2026, 10, 4));
    }
}
