use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// --- Employee Deduction Assignment (template assign + per-rule override) ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmployeeDeduction {
    pub id: Uuid,
    pub employee_id: i32,
    pub template_id: Option<Uuid>,
    pub kind: String, // permit | sick | absent | late
    pub grace_minutes: Option<i32>,
    pub fixed_amount: Option<i64>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AssignEmployeeDeductionRequest {
    pub employee_id: i32,
    pub template_id: Option<Uuid>,
    pub kind: String,
    pub grace_minutes: Option<i32>,
    pub fixed_amount: Option<i64>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmployeeDeductionRequest {
    pub template_id: Option<Uuid>,
    pub grace_minutes: Option<i32>,
    pub fixed_amount: Option<i64>,
    pub is_active: Option<bool>,
}

// --- Employee Statutory (BPJS/tax per-employee percent, NULL = company fallback) ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmployeeStatutory {
    pub employee_id: i32,
    pub bpjs_jht_employee: Option<f32>,
    pub bpjs_jp_employee: Option<f32>,
    pub bpjs_kes_employee: Option<f32>,
    pub pph21_borne_by_company: bool,
    pub ptkp_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertEmployeeStatutoryRequest {
    pub employee_id: i32,
    pub bpjs_jht_employee: Option<f32>,
    pub bpjs_jp_employee: Option<f32>,
    pub bpjs_kes_employee: Option<f32>,
    pub pph21_borne_by_company: Option<bool>,
    pub ptkp_status: Option<String>,
}

// --- Resolved statutory (employee percent or company fallback) ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedStatutory {
    pub employee_id: i32,
    pub bpjs_jht_employee: f32,
    pub bpjs_jp_employee: f32,
    pub bpjs_kes_employee: f32,
    pub bpjs_kes_wage_cap: i64,
    pub pph21_borne_by_company: bool,
    pub ptkp_status: String,
}

// --- Effective deduction rule (override wins, else linked template, else none) ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveDeductionRule {
    pub kind: String,
    pub grace_minutes: i32,
    pub fixed_amount: i64,
    pub source: String, // override | template | none
    pub template_id: Option<Uuid>,
}

// --- Effective date-scoped salary row (for calc engine join) ---
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EffectiveSalaryRow {
    pub employee_id: i32,
    pub salary_type: String,
    pub monthly_rate: i64,
    pub daily_salary: i64,
    pub hourly_rate: i64,
    pub effective_date: NaiveDate,
}
