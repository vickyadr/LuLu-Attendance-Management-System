/// Template rules opsi A: salary_templates carries rules_json, update cascades
/// rules-only to linked salaries (amounts untouched).
use lulu_attendance_server::models::payroll::{
    CreateSalaryTemplateRequest, UpdateSalaryTemplateRequest,
};

#[test]
fn template_create_accepts_rules_json() {
    let payload = serde_json::json!({
        "template_name": "Staff",
        "basic_salary_type": "monthly",
        "rules_json": [
            {"name": "Gaji", "function": "Add", "add": true, "deduct": false, "parameter": "=monthly_rate", "daily": false}
        ]
    });
    let req: CreateSalaryTemplateRequest =
        serde_json::from_value(payload).expect("template create deserializes");
    let rules = req.rules_json.expect("rules_json present");
    assert_eq!(rules.as_array().unwrap().len(), 1);
}

#[test]
fn template_create_without_rules_stays_none() {
    let payload = serde_json::json!({
        "template_name": "Staff",
        "basic_salary_type": "monthly"
    });
    let req: CreateSalaryTemplateRequest =
        serde_json::from_value(payload).expect("template create deserializes");
    assert!(req.rules_json.is_none(), "rules optional for legacy callers");
}

#[test]
fn template_update_accepts_rules_json() {
    let payload = serde_json::json!({
        "rules_json": [
            {"name": "Telat", "function": "Deduct", "add": false, "deduct": true, "parameter": "=late*15000", "daily": false}
        ]
    });
    let req: UpdateSalaryTemplateRequest =
        serde_json::from_value(payload).expect("template update deserializes");
    let rules = req.rules_json.expect("rules_json present");
    assert_eq!(rules.as_array().unwrap().len(), 1);
}
