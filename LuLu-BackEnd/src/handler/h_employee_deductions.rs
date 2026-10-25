use actix_web::web::ReqData;
use actix_web::{Responder, delete, get, post, put, web};
use chrono::NaiveDate;

use crate::models::payroll::DeductionTemplate;
use crate::models::payroll_employee_deductions::*;
use crate::utility::{
    authorization::{Action, check_action_access},
    stor::{AppState, GenericResponse, IntoResponse, NoDataResponse},
};

const ASSIGN_KINDS: [&str; 4] = ["permit", "sick", "absent", "late"];
const PTKP_STATUSES: [&str; 7] = ["TK", "K", "K1", "K2", "K3", "I0", "I1"];

fn valid_assign_input(
    kind: &str,
    grace: Option<i32>,
    amount: Option<i64>,
) -> Option<actix_web::HttpResponse> {
    if !ASSIGN_KINDS.contains(&kind) {
        return Some(
            NoDataResponse::new(
                "Invalid kind (permit | sick | absent | late)".to_string(),
                400,
            )
            .into_response(),
        );
    }
    if let Some(g) = grace {
        if g < 0 {
            return Some(
                NoDataResponse::new("grace_minutes must be >= 0".to_string(), 400).into_response(),
            );
        }
        if g > 0 && kind != "late" {
            return Some(
                NoDataResponse::new("grace_minutes only applies to kind=late".to_string(), 400)
                    .into_response(),
            );
        }
    }
    if let Some(a) = amount {
        if a < 0 {
            return Some(
                NoDataResponse::new("fixed_amount must be >= 0".to_string(), 400).into_response(),
            );
        }
    }
    None
}

fn valid_statutory_input(
    jht: Option<f32>,
    jp: Option<f32>,
    kes: Option<f32>,
    ptkp: &Option<String>,
) -> Option<actix_web::HttpResponse> {
    for (label, v) in [
        ("bpjs_jht_employee", jht),
        ("bpjs_jp_employee", jp),
        ("bpjs_kes_employee", kes),
    ] {
        if let Some(rate) = v {
            if !(0.0..=100.0).contains(&rate) {
                return Some(
                    NoDataResponse::new(format!("{} must be 0..100", label), 400).into_response(),
                );
            }
        }
    }
    if let Some(s) = ptkp {
        if !PTKP_STATUSES.contains(&s.as_str()) {
            return Some(
                NoDataResponse::new(
                    "Invalid ptkp_status (TK | K | K1 | K2 | K3 | I0 | I1)".to_string(),
                    400,
                )
                .into_response(),
            );
        }
    }
    None
}

// POST /api/payroll/employee-deductions — assign template to employee per kind, optional override
#[post("/payroll/employee-deductions")]
pub async fn assign_employee_deduction(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<AssignEmployeeDeductionRequest>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Add).await {
        return resp;
    }
    if let Some(err) = valid_assign_input(&body.kind, body.grace_minutes, body.fixed_amount) {
        return err;
    }
    if let Some(tid) = &body.template_id {
        let exists: Option<DeductionTemplate> =
            sqlx::query_as("SELECT * FROM deduction_templates WHERE id = $1")
                .bind(tid)
                .fetch_optional(&pool.db)
                .await
                .unwrap_or(None);
        if exists.is_none() {
            return NoDataResponse::new("Deduction template not found".to_string(), 404)
                .into_response();
        }
    }
    match sqlx::query_as::<_, EmployeeDeduction>(
        r#"INSERT INTO employee_deductions (employee_id, template_id, kind, grace_minutes, fixed_amount, is_active)
           VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (employee_id, kind) DO UPDATE SET
             template_id = EXCLUDED.template_id,
             grace_minutes = EXCLUDED.grace_minutes,
             fixed_amount = EXCLUDED.fixed_amount,
             is_active = EXCLUDED.is_active,
             updated_at = NOW()
           RETURNING *"#,
    )
    .bind(body.employee_id)
    .bind(&body.template_id)
    .bind(&body.kind)
    .bind(body.grace_minutes)
    .bind(body.fixed_amount)
    .bind(body.is_active.unwrap_or(true))
    .fetch_one(&pool.db)
    .await
    {
        Ok(row) => GenericResponse::ok(vec![row], "Employee deduction assigned".to_string()).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// GET /api/payroll/employee-deductions?employee_id=
#[get("/payroll/employee-deductions")]
pub async fn list_employee_deductions(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::View).await {
        return resp;
    }
    let emp = query.get("employee_id").and_then(|v| v.parse::<i32>().ok());
    match sqlx::query_as::<_, EmployeeDeduction>(
        "SELECT * FROM employee_deductions WHERE ($1::INT IS NULL OR employee_id = $1) ORDER BY employee_id ASC, kind ASC",
    )
    .bind(emp)
    .fetch_all(&pool.db)
    .await
    {
        Ok(rows) => GenericResponse::ok(rows, "Employee deductions retrieved".to_string()).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// PUT /api/payroll/employee-deductions/{id} — per-rule override update
#[put("/payroll/employee-deductions/{id}")]
pub async fn update_employee_deduction(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<uuid::Uuid>,
    body: web::Json<UpdateEmployeeDeductionRequest>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Edit).await {
        return resp;
    }
    let row_id = id.into_inner();
    let cur: Option<EmployeeDeduction> =
        sqlx::query_as("SELECT * FROM employee_deductions WHERE id = $1")
            .bind(&row_id)
            .fetch_optional(&pool.db)
            .await
            .unwrap_or(None);
    let Some(cur) = cur else {
        return NoDataResponse::new("Employee deduction not found".to_string(), 404)
            .into_response();
    };
    let grace = body.grace_minutes.or(cur.grace_minutes);
    let amount = body.fixed_amount.or(cur.fixed_amount);
    if let Some(err) = valid_assign_input(&cur.kind, grace, amount) {
        return err;
    }
    if let Some(tid) = body.template_id.as_ref() {
        let exists: Option<DeductionTemplate> =
            sqlx::query_as("SELECT * FROM deduction_templates WHERE id = $1")
                .bind(tid)
                .fetch_optional(&pool.db)
                .await
                .unwrap_or(None);
        if exists.is_none() {
            return NoDataResponse::new("Deduction template not found".to_string(), 404)
                .into_response();
        }
    }
    match sqlx::query_as::<_, EmployeeDeduction>(
        r#"UPDATE employee_deductions SET
             template_id = COALESCE($1, template_id),
             grace_minutes = $2,
             fixed_amount = $3,
             is_active = COALESCE($4, is_active),
             updated_at = NOW()
           WHERE id = $5 RETURNING *"#,
    )
    .bind(&body.template_id)
    .bind(grace)
    .bind(amount)
    .bind(body.is_active)
    .bind(&row_id)
    .fetch_optional(&pool.db)
    .await
    {
        Ok(Some(row)) => {
            GenericResponse::ok(vec![row], "Employee deduction updated".to_string()).into_response()
        }
        Ok(None) => {
            NoDataResponse::new("Employee deduction not found".to_string(), 404).into_response()
        }
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// DELETE /api/payroll/employee-deductions/{id}
#[delete("/payroll/employee-deductions/{id}")]
pub async fn delete_employee_deduction(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<uuid::Uuid>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Delete).await
    {
        return resp;
    }
    match sqlx::query("DELETE FROM employee_deductions WHERE id = $1")
        .bind(id.into_inner())
        .execute(&pool.db)
        .await
    {
        Ok(r) if r.rows_affected() > 0 => {
            NoDataResponse::new("Employee deduction deleted".to_string(), 200).into_response()
        }
        Ok(_) => {
            NoDataResponse::new("Employee deduction not found".to_string(), 404).into_response()
        }
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// PUT /api/payroll/employee-statutory — upsert per-employee BPJS/tax rates (NULL = company fallback)
#[put("/payroll/employee-statutory")]
pub async fn upsert_employee_statutory(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<UpsertEmployeeStatutoryRequest>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Edit).await {
        return resp;
    }
    if let Some(err) = valid_statutory_input(
        body.bpjs_jht_employee,
        body.bpjs_jp_employee,
        body.bpjs_kes_employee,
        &body.ptkp_status,
    ) {
        return err;
    }
    match sqlx::query_as::<_, EmployeeStatutory>(
        r#"INSERT INTO employee_statutory
             (employee_id, bpjs_jht_employee, bpjs_jp_employee, bpjs_kes_employee, pph21_borne_by_company, ptkp_status)
           VALUES ($1, $2, $3, $4, COALESCE($5, FALSE), COALESCE($6, 'TK'))
           ON CONFLICT (employee_id) DO UPDATE SET
             bpjs_jht_employee = EXCLUDED.bpjs_jht_employee,
             bpjs_jp_employee = EXCLUDED.bpjs_jp_employee,
             bpjs_kes_employee = EXCLUDED.bpjs_kes_employee,
             pph21_borne_by_company = EXCLUDED.pph21_borne_by_company,
             ptkp_status = EXCLUDED.ptkp_status,
             updated_at = NOW()
           RETURNING *"#,
    )
    .bind(body.employee_id)
    .bind(body.bpjs_jht_employee)
    .bind(body.bpjs_jp_employee)
    .bind(body.bpjs_kes_employee)
    .bind(body.pph21_borne_by_company)
    .bind(body.ptkp_status.clone())
    .fetch_one(&pool.db)
    .await
    {
        Ok(row) => GenericResponse::ok(vec![row], "Employee statutory saved".to_string()).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// GET /api/payroll/employee-statutory?employee_id= — resolved rates with company fallback
#[get("/payroll/employee-statutory")]
pub async fn get_employee_statutory(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::View).await {
        return resp;
    }
    let Some(emp) = query.get("employee_id").and_then(|v| v.parse::<i32>().ok()) else {
        return NoDataResponse::new("employee_id query param required".to_string(), 400)
            .into_response();
    };
    match resolve_statutory(&pool.db, emp).await {
        Ok(resolved) => {
            GenericResponse::ok(vec![resolved], "Employee statutory resolved".to_string())
                .into_response()
        }
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// Resolve per-employee statutory: employee percent wins, NULL falls back to payroll_config.
pub async fn resolve_statutory(
    db: &sqlx::PgPool,
    employee_id: i32,
) -> Result<ResolvedStatutory, sqlx::Error> {
    use sqlx::Row;
    let row: Option<EmployeeStatutory> =
        sqlx::query_as("SELECT * FROM employee_statutory WHERE employee_id = $1")
            .bind(employee_id)
            .fetch_optional(db)
            .await?;
    let cfg = sqlx::query(
        r#"SELECT bpjs_jht_employee::REAL AS jht, bpjs_jp_employee::REAL AS jp,
                  bpjs_kes_employee::REAL AS kes, bpjs_kes_wage_cap FROM payroll_config LIMIT 1"#,
    )
    .fetch_one(db)
    .await?;
    let cfg_jht: f32 = cfg.try_get("jht")?;
    let cfg_jp: f32 = cfg.try_get("jp")?;
    let cfg_kes: f32 = cfg.try_get("kes")?;
    let cfg_cap: i64 = cfg.try_get("bpjs_kes_wage_cap")?;
    let empty = EmployeeStatutory {
        employee_id,
        bpjs_jht_employee: None,
        bpjs_jp_employee: None,
        bpjs_kes_employee: None,
        pph21_borne_by_company: false,
        ptkp_status: "TK".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let emp = row.unwrap_or(empty);
    Ok(ResolvedStatutory {
        employee_id,
        bpjs_jht_employee: emp.bpjs_jht_employee.unwrap_or(cfg_jht),
        bpjs_jp_employee: emp.bpjs_jp_employee.unwrap_or(cfg_jp),
        bpjs_kes_employee: emp.bpjs_kes_employee.unwrap_or(cfg_kes),
        bpjs_kes_wage_cap: cfg_cap,
        pph21_borne_by_company: emp.pph21_borne_by_company,
        ptkp_status: emp.ptkp_status,
    })
}

// Effective deduction rules for one employee: override wins, else linked template, else none.
// Shared helper for calc engine (t_841599ab) — pure read, no writes.
pub async fn effective_deduction_rules(
    db: &sqlx::PgPool,
    employee_id: i32,
) -> Result<Vec<EffectiveDeductionRule>, sqlx::Error> {
    use sqlx::Row;
    let db_rows = sqlx::query(
        r#"SELECT ed.kind AS kind,
                  ed.grace_minutes AS grace_override,
                  ed.fixed_amount AS amount_override,
                  ed.template_id AS template_id,
                  dt.grace_minutes AS tpl_grace,
                  dt.fixed_amount AS tpl_amount
           FROM employee_deductions ed
           LEFT JOIN deduction_templates dt ON dt.id = ed.template_id
           WHERE ed.employee_id = $1 AND ed.is_active = TRUE"#,
    )
    .bind(employee_id)
    .fetch_all(db)
    .await?;
    let mut out = Vec::with_capacity(db_rows.len());
    for r in db_rows {
        let kind: String = r.try_get("kind")?;
        let grace_override: Option<i32> = r.try_get("grace_override")?;
        let amount_override: Option<i64> = r.try_get("amount_override")?;
        let template_id: Option<uuid::Uuid> = r.try_get("template_id")?;
        let tpl_grace: Option<i32> = r.try_get("tpl_grace")?;
        let tpl_amount: Option<i64> = r.try_get("tpl_amount")?;
        let (grace, amount, source) = match (grace_override, amount_override) {
            (Some(g), Some(a)) => (g, a, "override"),
            (Some(g), None) => (g, tpl_amount.unwrap_or(0), "override"),
            (None, Some(a)) => (tpl_grace.unwrap_or(0), a, "override"),
            (None, None) => match (tpl_grace, tpl_amount) {
                (Some(g), Some(a)) => (g, a, "template"),
                _ => (0, 0, "none"),
            },
        };
        out.push(EffectiveDeductionRule {
            kind,
            grace_minutes: grace,
            fixed_amount: amount,
            source: source.to_string(),
            template_id,
        });
    }
    Ok(out)
}

// Effective salary row as of a date: latest salaries row with effective_date <= date.
pub async fn effective_salary_as_of(
    db: &sqlx::PgPool,
    employee_id: i32,
    as_of: NaiveDate,
) -> Result<Option<EffectiveSalaryRow>, sqlx::Error> {
    sqlx::query_as::<_, EffectiveSalaryRow>(
        r#"SELECT employee_id, salary_type, monthly_rate, daily_salary, hourly_rate, effective_date
           FROM salaries WHERE employee_id = $1 AND effective_date <= $2
           ORDER BY effective_date DESC LIMIT 1"#,
    )
    .bind(employee_id)
    .bind(as_of)
    .fetch_optional(db)
    .await
}
