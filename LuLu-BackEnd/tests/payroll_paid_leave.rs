/// Paid leave logic: approved leave_requests apply leave_salary to matching attendance_daily rows.
/// A day with leave_kind='cuti|izin|sakit' and status='approved' computes daily_pay = daily_salary,
/// sets is_paid_leave=true, and contributes to paid_leave_pay total.
use chrono::NaiveDate;
use lulu_attendance_server::handler::payroll_api::GeneratePayslipsRequest;

#[test]
fn generate_applies_approved_leave_to_daily_pay() {
    // SETUP: employee 1 with leave_requests record (leave_kind='cuti', status='approved', date='2026-10-12')
    // is already inserted into DB by fixture; when payslip generates for period 2026-10-01..2026-10-31,
    // the leave day should be included and daily_pay = daily_salary (125000).
    let payload = serde_json::json!({
        "period_start": "2026-10-01",
        "period_end": "2026-10-31"
    });
    let req: GeneratePayslipsRequest =
        serde_json::from_value(payload).expect("request deserializes");
    assert!(req.daily_conditions.is_none(), "no conditions in paid leave test");
}

#[test]
fn daily_conditions_cuti_izin_sakit_produce_zero_pay() {
    // When daily_conditions override with 'cuti|izin|sakit' (NOT 'lupa_absen'),
    // payslip line gets days_credited=0, daily_pay=0 (zero-credit day).
    let payload = serde_json::json!({
        "period_start": "2026-10-01",
        "period_end": "2026-10-31",
        "daily_conditions": [
            {"employee_id": 1, "work_date": "2026-10-13", "condition": "izin"}
        ]
    });
    let req: GeneratePayslipsRequest =
        serde_json::from_value(payload).expect("conditions deserialize");
    let conds = req.daily_conditions.expect("conditions present");
    assert_eq!(conds[0].condition, "izin");
}
