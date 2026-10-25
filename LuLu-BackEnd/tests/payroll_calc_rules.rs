//! Rules-only pay calculation (BE3): sequential salary mutation, daily loop.
//! No payroll_config, BPJS, PPh 21, OT tier, or holiday lookup here:
//! every number comes from `salaries.rules_json` formulas.
use chrono::NaiveDate;
use lulu_attendance_server::handler::payroll_calc_rules::{
    DayInput, EmployeePayInput, SalaryRates, calc_employee_pay,
};
use lulu_attendance_server::models::payroll::SalaryRule;

fn day(date: u32, present: bool) -> DayInput {
    DayInput {
        date: NaiveDate::from_ymd_opt(2026, 10, date).expect("valid test date"),
        ot_hours: 0.0,
        late_minutes: 0.0,
        is_sick: false,
        is_permission: false,
        is_holiday: false,
        present,
        day_weight: 1.0,
        paid_leave: false,
        skip: false,
    }
}

fn add_rule(name: &str, function: &str, daily: bool) -> SalaryRule {
    SalaryRule {
        name: name.to_string(),
        function: function.to_string(),
        add: true,
        deduct: false,
        parameter: None,
        daily,
    }
}

fn deduct_rule(name: &str, parameter: &str, daily: bool) -> SalaryRule {
    SalaryRule {
        name: name.to_string(),
        function: "Deduct".to_string(),
        add: false,
        deduct: true,
        parameter: Some(parameter.to_string()),
        daily,
    }
}

fn rates() -> SalaryRates {
    SalaryRates {
        monthly_rate: 6_000_000.0,
        daily_salary: 300_000.0,
        hourly_rate: 35_000.0,
    }
}

#[test]
fn daily_add_rule_counts_present_days_only() {
    let input = EmployeePayInput {
        rates: rates(),
        rules: vec![add_rule("daily wage", "present * daily_rate", true)],
        days: vec![day(1, true), day(2, true), day(3, false)],
    };
    let result = calc_employee_pay(&input).expect("calc succeeds");
    assert_eq!(result.gross_adds as i64, 600_000);
    assert_eq!(result.total_deducts as i64, 0);
    assert_eq!(result.present_days, 2);
    assert_eq!(result.absent_days, 1);
    assert_eq!(result.days.len(), 3);
}

#[test]
fn sequential_rules_see_mutated_salary() {
    // Second rule reads salary AFTER the first rule added to it.
    let input = EmployeePayInput {
        rates: rates(),
        rules: vec![
            add_rule("base", "1000", false),
            add_rule("ten percent of mutated", "salary * 0.1", false),
        ],
        days: vec![day(1, true)],
    };
    let result = calc_employee_pay(&input).expect("calc succeeds");
    // salary starts at 6_000_000, +1000 = 6_001_000, 10% = 600_100.
    assert_eq!(result.gross_adds as i64, 601_100);
}

#[test]
fn deduct_rule_shrinks_salary() {
    // FE 3-col shape: function holds the kind label, formula in parameter.
    let input = EmployeePayInput {
        rates: rates(),
        rules: vec![
            add_rule("base", "1000", false),
            deduct_rule("fee", "=100", false),
        ],
        days: vec![day(1, true)],
    };
    let result = calc_employee_pay(&input).expect("calc succeeds");
    assert_eq!(result.gross_adds as i64, 1000);
    assert_eq!(result.total_deducts as i64, 100);
}

#[test]
fn skipped_day_contributes_nothing() {
    let skip = DayInput {
        skip: true,
        ..day(1, false)
    };
    let input = EmployeePayInput {
        rates: rates(),
        rules: vec![add_rule("daily wage", "present * daily_rate", true)],
        days: vec![skip, day(2, true)],
    };
    let result = calc_employee_pay(&input).expect("calc succeeds");
    assert_eq!(result.gross_adds as i64, 300_000);
    assert_eq!(result.present_days, 1);
    assert_eq!(result.absent_days, 0);
    assert_eq!(result.days.len(), 2);
}

#[test]
fn bad_formula_fails_with_rule_name() {
    let input = EmployeePayInput {
        rates: rates(),
        rules: vec![add_rule("broken", "present *", true)],
        days: vec![day(1, true)],
    };
    let error = calc_employee_pay(&input).expect_err("bad formula fails");
    assert!(error.contains("broken"), "error names the rule: {error}");
}
