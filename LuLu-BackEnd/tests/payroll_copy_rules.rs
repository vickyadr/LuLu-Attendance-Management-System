/// Copy rules endpoint (BE2c): POST /api/payroll/employee-salary-copy-rules
/// copies rules_json from one employee salary row to another.
/// Single handler `copy_salary_rules` + flat route (avoids clash with
/// GET /employee-salary/{employee_id}) + this test.
use lulu_attendance_server::models::payroll::CopySalaryRulesRequest;

#[test]
fn copy_rules_request_deserializes_employee_ids() {
    let payload = serde_json::json!({
        "from_employee_id": 3,
        "to_employee_id": 7
    });
    let req: CopySalaryRulesRequest =
        serde_json::from_value(payload).expect("copy rules request deserializes");
    assert_eq!(req.from_employee_id, 3);
    assert_eq!(req.to_employee_id, 7);
}

#[test]
fn copy_rules_request_rejects_missing_target() {
    let payload = serde_json::json!({ "from_employee_id": 3 });
    assert!(
        serde_json::from_value::<CopySalaryRulesRequest>(payload).is_err(),
        "to_employee_id is required"
    );
}

#[test]
fn copy_rules_same_id_shape_reaches_handler_guard() {
    // Same-id pair deserializes fine; the handler rejects it with 400
    // (from_employee_id and to_employee_id must differ).
    let payload = serde_json::json!({
        "from_employee_id": 3,
        "to_employee_id": 3
    });
    let req: CopySalaryRulesRequest =
        serde_json::from_value(payload).expect("same-id shape deserializes");
    assert_eq!(req.from_employee_id, req.to_employee_id);
}
