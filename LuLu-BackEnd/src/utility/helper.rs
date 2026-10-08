use chrono::{DateTime, Datelike, Timelike};

/// Instant for `seconds`, falling back to the epoch when it is unrepresentable.
fn at(seconds: i64) -> DateTime<chrono::Utc> {
    DateTime::from_timestamp(seconds, 0).unwrap_or(DateTime::from_timestamp(0, 0).unwrap())
}

/// Midnight of the day containing `seconds`, offset by `tz + hour` hours.
pub fn ts_at_zero(seconds: i64, tz: i32) -> i64 {
    ts_at_hour(seconds, tz, 0)
}

pub fn ts_at_hour(seconds: i64, tz: i32, hour: i32) -> i64 {
    let t = at(seconds);
    let h: i64 = (t.time().hour() * 3600) as i64;
    let m: i64 = (t.time().minute() * 60) as i64;
    let s: i64 = (t.time().second()) as i64;
    t.timestamp() - h - m - s + (tz as i64 + hour as i64) * 3600
}

pub fn ts_dow(seconds: i64, tz: i32) -> u32 {
    let shifted = seconds.saturating_add((tz as i64).saturating_mul(3600));
    DateTime::from_timestamp(shifted, 0)
        .unwrap_or_else(|| at(0))
        .weekday()
        .number_from_monday()
}

pub fn ts_week(seconds: i64, tz: i32) -> u32 {
    let shifted = seconds.saturating_add((tz as i64).saturating_mul(3600));
    DateTime::from_timestamp(shifted, 0)
        .unwrap_or_else(|| at(0))
        .iso_week()
        .week()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-10-08T00:00:00Z — the report day the E2E script uses.
    const DAY: i64 = 1_791_417_600;

    /// 2026-10-08 is a Thursday, ISO week 41.
    const DOW: u32 = 4;
    const WEEK: u32 = 41;

    #[test]
    fn ts_at_zero_returns_utc_midnight() {
        assert_eq!(ts_at_zero(DAY + 3_600 * 10, 0), DAY);
        assert_eq!(ts_at_zero(DAY + 3_600 * 23 + 3_599, 0), DAY);
        assert_eq!(ts_at_zero(DAY, 0), DAY);
    }

    #[test]
    fn ts_at_hour_lands_on_the_requested_hour() {
        for hour in [0, 6, 12, 23] {
            let at_hour = ts_at_hour(DAY + 3_600 * 13, 0, hour);
            let midnight = ts_at_hour(at_hour, 0, 0);
            assert_eq!(midnight, DAY, "hour {hour} escaped its day");
            assert_eq!((at_hour - midnight) / 3_600, hour as i64);
        }
    }

    #[test]
    fn dow_and_week_are_monday_based_iso_values() {
        assert_eq!(ts_dow(DAY, 0), DOW);
        assert_eq!(ts_week(DAY, 0), WEEK);
        // The next day is a Friday.
        assert_eq!(ts_dow(DAY + 86_400, 0), DOW + 1);
    }

    #[test]
    fn dow_and_week_stay_in_range_across_a_long_span() {
        for d in 0..800 {
            let dow = ts_dow(DAY + 86_400 * d, 0);
            assert!((1..=7).contains(&dow), "day {d} gave dow {dow}");
            let week = ts_week(DAY + 86_400 * d, 0);
            assert!((1..=53).contains(&week), "day {d} gave week {week}");
        }
    }

    #[test]
    fn out_of_range_timestamps_do_not_panic() {
        for bad in [
            i64::MAX,
            i64::MIN,
            -1,
            1,
            i64::MAX / 2,
            -(i64::MAX / 2),
            i64::MAX - 3_600,
        ] {
            let _ = ts_at_zero(bad, 0);
            let _ = ts_at_hour(bad, 0, 12);
            let _ = ts_at_hour(bad, 7, 23);
            let _ = ts_dow(bad, 0);
            let _ = ts_dow(bad, -7);
            let _ = ts_week(bad, 0);
            let _ = ts_week(bad, 14);
        }
    }

    #[test]
    fn tz_offset_is_added_after_truncating_the_utc_day() {
        assert_eq!(ts_at_hour(DAY, 7, 0), DAY + 7 * 3_600);
        assert_eq!(ts_at_hour(DAY, -7, 0), DAY - 7 * 3_600);
    }
}
