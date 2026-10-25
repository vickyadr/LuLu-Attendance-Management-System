use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;

// --- Payroll Config ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PayrollConfig {
    pub id: Uuid,
    pub work_week_days: i32,
    pub daily_work_hours: i32,
    pub ot_daily_cap_hours: i32,
    pub ot_weekly_cap_hours: i32,
    pub ot_rate_first_hour: f32,
    pub ot_rate_subsequent: f32,
    pub rest_day_multiplier: f32,
    pub ot_day_threshold_hours: i32,
    pub ot_below_mode: String, // flat | tiered
    pub ot_tiers_json: serde_json::Value, // [{up_to_hours, multiplier}]
    pub holiday_multiplier: f32,
    pub holiday_basis: String, // hourly | daily
    pub carry_unapproved: String, // never | next | backpay
    pub bpjs_jkk_rate_min: f32,
    pub bpjs_jkk_rate_max: f32,
    pub bpjs_jkm_rate: f32,
    pub bpjs_jht_employer: f32,
    pub bpjs_jht_employee: f32,
    pub bpjs_jp_employer: f32,
    pub bpjs_jp_employee: f32,
    pub bpjs_kes_employer: f32,
    pub bpjs_kes_employee: f32,
    pub bpjs_kes_wage_cap: i64,
    pub meal_ot_threshold_hours: i32,
    pub meal_calories: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePayrollConfigRequest {
    pub work_week_days: Option<i32>,
    pub daily_work_hours: Option<i32>,
    pub ot_daily_cap_hours: Option<i32>,
    pub ot_weekly_cap_hours: Option<i32>,
    pub rest_day_multiplier: Option<f32>,
    pub ot_day_threshold_hours: Option<i32>,
    pub ot_below_mode: Option<String>,
    pub ot_tiers_json: Option<serde_json::Value>,
    pub holiday_multiplier: Option<f32>,
    pub holiday_basis: Option<String>,
    pub carry_unapproved: Option<String>,
    pub bpjs_jkk_rate_min: Option<f32>,
    pub bpjs_jkk_rate_max: Option<f32>,
    pub bpjs_jkm_rate: Option<f32>,
    pub meal_ot_threshold_hours: Option<i32>,
}

// --- Salary Templates ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalaryTemplate {
    pub id: Uuid,
    pub template_name: String,
    pub description: Option<String>,
    pub basic_salary_type: String, // monthly | hourly | daily
    pub hra_percent: Option<f32>,
    pub hra_fixed: Option<i64>,
    pub transport_fixed: Option<i64>,
    pub monthly_rate: Option<i64>,
    pub daily_rate: Option<i64>,
    pub hourly_rate: Option<i64>,
    pub allowances_json: serde_json::Value,
    pub deductions_json: serde_json::Value,
    pub rules_json: serde_json::Value, // live rules (opsi A): linked salaries follow on edit
    pub ot_eligible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSalaryTemplateRequest {
    pub template_name: String,
    pub description: Option<String>,
    pub basic_salary_type: String,
    pub hra_percent: Option<f32>,
    pub hra_fixed: Option<i64>,
    pub transport_fixed: Option<i64>,
    pub monthly_rate: Option<i64>,
    pub daily_rate: Option<i64>,
    pub hourly_rate: Option<i64>,
    pub allowances_json: Option<serde_json::Value>,
    pub deductions_json: Option<serde_json::Value>,
    pub rules_json: Option<serde_json::Value>,
    pub ot_eligible: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSalaryTemplateRequest {
    pub description: Option<String>,
    pub hra_percent: Option<f32>,
    pub hra_fixed: Option<i64>,
    pub transport_fixed: Option<i64>,
    pub monthly_rate: Option<i64>,
    pub daily_rate: Option<i64>,
    pub hourly_rate: Option<i64>,
    pub allowances_json: Option<serde_json::Value>,
    pub deductions_json: Option<serde_json::Value>,
    pub rules_json: Option<serde_json::Value>,
    pub ot_eligible: Option<bool>,
}

// --- Employee Salary ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmployeeSalary {
    pub id: Uuid,
    pub employee_id: i32,
    pub template_id: Option<Uuid>,
    pub salary_type: String, // monthly | daily
    pub monthly_rate: i64,
    pub daily_salary: i64, // canonical per-day rate: manual input (was monthly/20)
    pub hourly_rate: i64,   // manual input (was monthly/173)
    pub basic_salary: i64,
    pub department: Option<String>, // snapshot at assign time
    pub rules_json: serde_json::Value, // [SalaryRule]: formula rules applied in order
    pub fixed_allowances_json: serde_json::Value,
    pub non_fixed_allowances_json: serde_json::Value,
    pub ot_calculation_basis: i32, // 100 or 75
    pub ot_eligible: bool,
    pub wage_cap: Option<i64>,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One formula pay rule inside `rules_json`.
/// `function` is a payroll_expr formula over KNOWN_VARS.
/// Exactly one of `add` / `deduct` must be true.
/// `parameter` is free text holding the formula, raw or `=` prefixed.
/// `daily` true = evaluated per workday, false = once per period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryRule {
    pub name: String,
    pub function: String,
    pub add: bool,
    pub deduct: bool,
    pub parameter: Option<String>,
    pub daily: bool,
}

impl SalaryRule {
    /// Effective formula: FE 3-col shape stores the kind label in
    /// `function` and the formula in `parameter`, so prefer parameter.
    /// Legacy rows carry the formula directly in `function`.
    pub fn expression(&self) -> &str {
        match &self.parameter {
            Some(param) if !param.trim().is_empty() => param,
            _ => &self.function,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployeeSalaryRequest {
    pub employee_id: i32,
    pub template_id: Option<Uuid>,
    pub salary_type: String, // monthly | daily
    pub monthly_rate: i64,
    pub daily_salary: i64, // manual per-day rate, no division
    pub hourly_rate: i64,   // manual hourly rate, no division
    pub basic_salary: i64,
    pub department: Option<String>,
    pub rules_json: Option<serde_json::Value>, // [SalaryRule]
    pub fixed_allowances_json: Option<serde_json::Value>,
    pub non_fixed_allowances_json: Option<serde_json::Value>,
    pub ot_calculation_basis: Option<i32>,
    pub ot_eligible: Option<bool>,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSalaryRulesRequest {
    pub rules_json: serde_json::Value, // [SalaryRule], replaces whole array
}

#[derive(Debug, Deserialize)]
pub struct CopySalaryRulesRequest {
    pub from_employee_id: i32,
    pub to_employee_id: i32,
}

// --- Public Holidays ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PublicHoliday {
    pub id: Uuid,
    pub holiday_date: NaiveDate,
    pub holiday_name: String,
    pub holiday_type: String, // official | bridge | custom
    pub ot_rate_multiplier_min: f32,
    pub ot_rate_multiplier_max: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePublicHolidayRequest {
    pub holiday_date: NaiveDate,
    pub holiday_name: String,
    pub holiday_type: Option<String>,
    pub ot_rate_multiplier_min: Option<f32>,
    pub ot_rate_multiplier_max: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePublicHolidayRequest {
    pub holiday_name: Option<String>,
    pub holiday_type: Option<String>,
    pub ot_rate_multiplier_min: Option<f32>,
    pub ot_rate_multiplier_max: Option<f32>,
}

// --- Daily Absences (pending missed-punch approvals) ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DailyAbsence {
    pub id: Uuid,
    pub employee_id: i32,
    pub absence_date: NaiveDate,
    pub absence_reason: String,
    pub status: String,
    pub recorded_by: String,
    pub approved_by: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DailyCondition {
    pub employee_id: i32,
    pub work_date: NaiveDate,
    pub condition: String,
}

// --- Response Types ---
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub message: String,
    pub data: T,
    pub code: u16,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(message: &str, data: T) -> Self {
        Self {
            message: message.to_string(),
            data,
            code: 200,
        }
    }

    pub fn created(message: &str, data: T) -> Self {
        Self {
            message: message.to_string(),
            data,
            code: 201,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub message: String,
    pub code: u16,
}

impl ApiError {
    pub fn bad_request(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            code: 400,
        }
    }

    pub fn not_found(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            code: 404,
        }
    }

    pub fn conflict(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            code: 409,
        }
    }

    pub fn internal_error(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            code: 500,
        }
    }
}

// --- Deduction Templates ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeductionTemplate {
    pub id: Uuid,
    pub template_name: String,
    pub description: Option<String>,
    pub kind: String, // permit | sick | absent | late
    pub policy: String, // standard | strict | lenient | custom
    pub grace_minutes: i32,
    pub fixed_amount: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeductionTemplateRequest {
    pub template_name: String,
    pub description: Option<String>,
    pub kind: String,
    pub policy: Option<String>,
    pub grace_minutes: Option<i32>,
    pub fixed_amount: Option<i64>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeductionTemplateRequest {
    pub description: Option<String>,
    pub kind: Option<String>,
    pub policy: Option<String>,
    pub grace_minutes: Option<i32>,
    pub fixed_amount: Option<i64>,
    pub is_active: Option<bool>,
}
