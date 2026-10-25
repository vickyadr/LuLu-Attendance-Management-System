//! Rules-only pay calculation: sequential salary mutation over a daily loop.
//!
//! Usage: build an [`EmployeePayInput`] (rates + ordered rules + one
//! [`DayInput`] per calendar day), call [`calc_employee_pay`]. Add rules
//! grow the running salary, deduct rules shrink it. Daily rules run once
//! per day, period rules run once at the end. No payroll_config, BPJS,
//! PPh 21, OT tier, or holiday table is consulted here; every number
//! comes from `salaries.rules_json` formulas evaluated with
//! [`crate::handler::payroll_expr::evaluate`].
//!
//! Day semantics: a day with an attendance row (or paid-leave mark) is
//! present, an explicit absent condition zeroes the day outright, any
//! other calendar day counts absent. Days nobody synced still count,
//! so formulas own the meaning of missing data. Requires Rust 1.70+
//! (std only, no dependencies).

use chrono::NaiveDate;
use std::collections::HashMap;

use crate::handler::payroll_expr::{PayrollVars, evaluate};
use crate::models::payroll::SalaryRule;

/// Manual rates from the employee salary row. All values in whole currency units.
#[derive(Debug, Clone, Default)]
pub struct SalaryRates {
    pub monthly_rate: f64,
    pub daily_salary: f64,
    pub hourly_rate: f64,
}

/// One calendar day of attendance context.
#[derive(Debug, Clone)]
pub struct DayInput {
    pub date: NaiveDate,
    /// Overtime minutes recorded for the day (0 when none).
    pub ot_hours: f64,
    /// Late minutes for the day: punch_in minus shift start, minus the
    /// report late tolerance. 0 when on time or no punch.
    pub late_minutes: f64,
    /// True when approved sick leave covers the day.
    pub is_sick: bool,
    /// True when approved non-sick leave (izin/cuti) covers the day.
    pub is_permission: bool,
    /// True when the day is a public holiday.
    pub is_holiday: bool,
    /// True when the day was worked (counts toward present).
    pub present: bool,
    /// 1.0 full day, 0.5 half day (single punch or late arrival past half shift).
    pub day_weight: f64,
    /// True when approved leave covers the day (not present, not absent).
    pub paid_leave: bool,
    /// True on explicit absent conditions: the day contributes nothing
    /// and is not counted (overridden, not worked).
    pub skip: bool,
}

/// Full input for one employee and one period.
#[derive(Debug, Clone)]
pub struct EmployeePayInput {
    pub rates: SalaryRates,
    /// Ordered rules from `salaries.rules_json`. Order is significant:
    /// later rules see the salary mutated by earlier ones.
    pub rules: Vec<SalaryRule>,
    /// One entry per calendar day in the period, ascending.
    pub days: Vec<DayInput>,
}

/// Per-day gross adds (rounded by the caller for storage).
#[derive(Debug, Clone)]
pub struct DayPay {
    pub date: NaiveDate,
    pub gross_add: f64,
    pub present: bool,
    /// 1.0 full day, 0.5 half day. Stored on the line so the payslip shows the
    /// same number the pay was computed from.
    pub weight: f64,
    pub ot_hours: f64,
}

/// Period result. All values raw f64; the caller rounds to i64.
#[derive(Debug, Clone)]
pub struct PayResult {
    /// Sum of every add rule (daily loop + period rules).
    pub gross_adds: f64,
    /// Sum of every deduct rule (daily loop + period rules).
    pub total_deducts: f64,
    pub days: Vec<DayPay>,
    pub present_days: i64,
    pub absent_days: i64,
    pub ot_total: f64,
}

fn base_vars(rates: &SalaryRates, salary: f64) -> HashMap<String, f64> {
    PayrollVars {
        salary: salary,
        monthly_rate: rates.monthly_rate,
        daily_rate: rates.daily_salary,
        hourly_rate: rates.hourly_rate,
        ..Default::default()
    }
    .to_map()
}

/// Evaluate one rule and fold its value into salary/adds/deducts.
/// Returns [`String`] error naming the rule on any failure.
fn apply_rule(
    rule: &SalaryRule,
    vars: &HashMap<String, f64>,
    salary: &mut f64,
    adds: &mut f64,
    deducts: &mut f64,
) -> Result<(), String> {
    let value = evaluate(rule.expression(), vars)
        .map_err(|error| format!("rule '{}': {error}", rule.name))?;
    if !value.is_finite() {
        return Err(format!("rule '{}': result is not finite", rule.name));
    }
    *salary += value;
    if rule.add {
        *adds += value;
    } else {
        *deducts += value;
    }
    Ok(())
}

/// Compute pay purely from rules: sequential salary mutation, daily loop.
///
/// Daily rules see per-day `overtime` in minutes;
/// period rules see the period total in minutes. Any evaluation failure, non-finite
/// result, or negative total aborts with a message naming the rule.
pub fn calc_employee_pay(input: &EmployeePayInput) -> Result<PayResult, String> {
    let mut salary = input.rates.monthly_rate;
    let mut gross_adds = 0.0;
    let mut total_deducts = 0.0;
    let mut ot_total = 0.0;
    let mut present_days = 0_i64;
    let mut absent_days = 0_i64;
    let mut sick_days = 0_i64;
    let mut permission_days = 0_i64;
    let mut late_days = 0_i64;
    let mut late_total = 0.0;
    let mut holiday_days = 0_i64;
    let mut days = Vec::with_capacity(input.days.len());

    let (daily_rules, period_rules): (Vec<&SalaryRule>, Vec<&SalaryRule>) = input
        .rules
        .iter()
        .partition(|rule| rule.daily);

    for day in &input.days {
        if day.skip {
            // Explicit absent condition: the day contributes nothing and
            // is not counted (overridden, not worked).
            days.push(DayPay {
                date: day.date,
                gross_add: 0.0,
                present: false,
                weight: 0.0,
                ot_hours: 0.0,
            });
            continue;
        }
        if day.is_holiday {
            holiday_days += 1;
        }
        let present = day.present && !day.paid_leave;
        let absent = !day.present && !day.paid_leave;
        // Half-day weight: single punch or arrival past half shift counts 0.5.
        let weight = if present { day.day_weight.clamp(0.0, 1.0) } else { 0.0 };
        if present {
            present_days += 1;
        } else if absent {
            absent_days += 1;
        }
        // Sick/permission count even when the day is skipped: approved
        // leave carries its own mark independent of the skip override.
        let sick = day.is_sick;
        let permission = day.is_permission;
        if sick {
            sick_days += 1;
        }
        if permission {
            permission_days += 1;
        }
        let late_min = day.late_minutes.max(0.0);
        let late_flag = if late_min > 0.0 { 1.0 } else { 0.0 };
        if late_min > 0.0 {
            late_days += 1;
        }
        late_total += late_min;
        ot_total += day.ot_hours.max(0.0);
        let mut vars = base_vars(&input.rates, salary);
        vars.insert("overtime".to_string(), day.ot_hours.max(0.0));
        vars.insert("present".to_string(), weight);
        vars.insert("absent".to_string(), if absent { 1.0 } else { 0.0 });
        vars.insert("sick".to_string(), if sick { 1.0 } else { 0.0 });
        vars.insert("permission".to_string(), if permission { 1.0 } else { 0.0 });
        vars.insert("late".to_string(), late_min);
        vars.insert("late_time".to_string(), late_min);
        vars.insert("late_day".to_string(), late_flag);
        vars.insert("dow".to_string(), day.date.format("%w").to_string().parse::<f64>().unwrap_or(0.0));
        vars.insert("hols".to_string(), if day.is_holiday { 1.0 } else { 0.0 });
        let adds_before = gross_adds;
        let deducts_before = total_deducts;
        for rule in &daily_rules {
            // Refresh salary-dependent vars so later rules see mutation.
            vars.insert("salary".to_string(), salary);
            apply_rule(rule, &vars, &mut salary, &mut gross_adds, &mut total_deducts)?;
        }
        let day_add = gross_adds - adds_before;
        let day_ded = total_deducts - deducts_before;
        if day_add - day_ded < 0.0 {
            return Err(format!("rule produced negative day total on {}", day.date));
        }
        days.push(DayPay {
            date: day.date,
            gross_add: day_add,
            present,
            weight,
            ot_hours: day.ot_hours.max(0.0),
        });
    }

    let mut vars = base_vars(&input.rates, salary);
    vars.insert("overtime".to_string(), ot_total);
    vars.insert("present".to_string(), present_days as f64);
    vars.insert("absent".to_string(), absent_days as f64);
    vars.insert("sick".to_string(), sick_days as f64);
    vars.insert("permission".to_string(), permission_days as f64);
    vars.insert("late".to_string(), late_total);
    vars.insert("late_time".to_string(), late_total);
    vars.insert("late_day".to_string(), late_days as f64);
    vars.insert("hols".to_string(), holiday_days as f64);
    // Day-of-week only makes sense per day: warn when a period rule uses it.
    for rule in &period_rules {
        let expr = rule.expression().to_lowercase();
        if expr.contains("dow") {
            return Err(format!(
                "rule '{}' uses 'dow' but is not daily-checked; tick Daily for day-of-week rules",
                rule.name
            ));
        }
    }
    for rule in &period_rules {
        vars.insert("salary".to_string(), salary);
        apply_rule(rule, &vars, &mut salary, &mut gross_adds, &mut total_deducts)?;
    }

    if !gross_adds.is_finite() || !total_deducts.is_finite() {
        return Err("period total is not finite".to_string());
    }
    if gross_adds < 0.0 || total_deducts < 0.0 {
        return Err("rule produced negative period total".to_string());
    }
    Ok(PayResult {
        gross_adds,
        total_deducts,
        days,
        present_days,
        absent_days,
        ot_total,
    })
}
