use chrono::NaiveDate;

pub fn validate_month_period(
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> Result<(), &'static str> {
    if period_start > period_end {
        return Err("Period start must be before or equal to end date");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn period_validation_accepts_any_date_range() {
        let start = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 11, 19).unwrap();
        // Accept custom payroll cycles (not just calendar months)
        assert_eq!(validate_month_period(start, end), Ok(()));

        // Accept single day
        assert_eq!(validate_month_period(start, start), Ok(()));

        // Reject reversed dates
        assert!(validate_month_period(end, start).is_err());
    }
}
