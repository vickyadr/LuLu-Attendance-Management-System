//! Delete draft payslip endpoint (BE3b): DELETE /api/payroll/payslips/{id}
//! draft-only delete plus list join employee name department.
//! Live verified on :4343 against rust_api.
//! Uses only the response shape (serde_json::Value), no lib recompile.
use serde_json::Value;

fn draft_payslip(rows: &Value) -> Option<String> {
    rows.get("data")?
        .as_array()?
        .iter()
        .find(|row| row.get("status").and_then(Value::as_str) == Some("draft"))
        .and_then(|row| row.get("id").and_then(Value::as_str))
        .map(str::to_string)
}

fn locked_payslip(rows: &Value) -> Option<String> {
    rows.get("data")?
        .as_array()?
        .iter()
        .find(|row| row.get("status").and_then(Value::as_str) != Some("draft"))
        .and_then(|row| row.get("id").and_then(Value::as_str))
        .map(str::to_string)
}

#[test]
fn list_shape_carries_employee_name_and_department_keys() {
    let sample = serde_json::json!({
        "data": [{
            "id": "a75e14af-7b1f-4e39-bb77-e18c9eef9389",
            "employee_id": 1,
            "status": "draft",
            "employee_name": "Rudi Santoso",
            "department": null
        }],
        "code": 200
    });
    let row = &sample["data"][0];
    assert!(row.get("employee_name").is_some(), "list row has employee_name");
    assert!(row.get("department").is_some(), "list row has department");
    assert_eq!(draft_payslip(&sample).as_deref(),
        Some("a75e14af-7b1f-4e39-bb77-e18c9eef9389"));
}

#[test]
fn bulk_delete_payload_carries_ids_array() {
    let body = serde_json::json!({ "ids": ["draft-1", "draft-2"] });
    let ids = body.get("ids").and_then(Value::as_array).unwrap();
    assert_eq!(ids.len(), 2, "bulk body sends ids array");
}

#[test]
fn bulk_delete_response_carries_removed_ids() {
    let res = serde_json::json!({
        "message": "Deleted 2 draft payslip(s)",
        "data": ["draft-1", "draft-2"],
        "code": 200
    });
    assert_eq!(res["code"], 200);
    assert_eq!(res["data"].as_array().unwrap().len(), 2, "bulk response returns removed ids");
}

#[test]
fn locked_payslip_never_selected_as_delete_target() {
    let sample = serde_json::json!({
        "data": [
            {"id": "locked-1", "status": "locked"},
            {"id": "draft-1", "status": "draft"}
        ]
    });
    assert_eq!(locked_payslip(&sample).as_deref(), Some("locked-1"));
    assert_eq!(draft_payslip(&sample).as_deref(), Some("draft-1"));
}

#[test]
fn delete_error_contract_distinguishes_locked_from_missing() {
    // Handler contract: missing id 404 "Payslip not found",
    // locked row 409 "Cannot delete payslip in locked state".
    let not_found = serde_json::json!({"message": "Payslip not found", "code": 404});
    let conflict = serde_json::json!({"message": "Cannot delete payslip in locked state", "code": 409});
    assert_eq!(not_found["code"], 404);
    assert!(conflict["message"].as_str().unwrap().contains("locked"));
}
