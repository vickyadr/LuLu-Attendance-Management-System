/// Generate accepts an optional `daily_conditions` array for on-demand payroll:
/// explicit per-day input (absent | cuti | izin | sakit | lupa_absen) that
/// overrides whatever attendance_daily holds for that employee and day.
/// lupa_absen records a zero-credit line plus a pending row in daily_absences.
use chrono::NaiveDate;
use lulu_attendance_server::handler::payroll_api::GeneratePayslipsRequest;

#[test]
fn generate_request_accepts_daily_conditions() {
    let payload = serde_json::json!({
        "period_start": "2026-10-01",
        "period_end": "2026-10-31",
        "daily_conditions": [
            {"employee_id": 1, "work_date": "2026-10-12", "condition": "lupa_absen"}
        ]
    });
    let req: GeneratePayslipsRequest =
        serde_json::from_value(payload).expect("daily_conditions must deserialize");
    let conds = req.daily_conditions.expect("conditions present");
    assert_eq!(conds.len(), 1);
    assert_eq!(conds[0].employee_id, 1);
    assert_eq!(
        conds[0].work_date,
        NaiveDate::from_ymd_opt(2026, 10, 12).unwrap()
    );
    assert_eq!(conds[0].condition, "lupa_absen");
}

#[test]
fn generate_request_without_conditions_stays_valid() {
    let payload = serde_json::json!({
        "period_start": "2026-10-01",
        "period_end": "2026-10-31"
    });
    let req: GeneratePayslipsRequest =
        serde_json::from_value(payload).expect("old shape still valid");
    assert!(req.daily_conditions.is_none());
}
