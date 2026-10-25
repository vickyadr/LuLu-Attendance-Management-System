use actix_web::{HttpResponse, Responder, delete, get, post, web};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    handler::payroll_calc_rules::{DayInput, EmployeePayInput, SalaryRates, calc_employee_pay},
    models::payroll::{ApiError, DailyAbsence, DailyCondition, SalaryRule},
    utility::{
        authorization::{Action, check_action_access, get_session_user},
        stor::{AppState, GenericResponse, IntoResponse, NoDataResponse},
    },
};
use actix_web::web::ReqData;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratePayslipsRequest {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    #[serde(default)]
    pub daily_conditions: Option<Vec<DailyCondition>>,
    // Scope filter: exactly one of employee_id or department may be set.
    #[serde(default)]
    pub employee_id: Option<i32>,
    #[serde(default)]
    pub department: Option<String>,
}

/// Payslip row plus the employee name and department snapshot (BE3).
/// `FromRow` reads the joined query below; flat so existing GET responses
/// keep their shape with two extra fields.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PayslipHeader {
    pub id: Uuid,
    pub employee_id: i32,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub regular_days_pay: i64,
    pub ot_pay: i64,
    pub holiday_premium_pay: i64,
    pub paid_leave_pay: i64,
    pub thr_amount: i64,
    pub other_earnings_json: Option<serde_json::Value>,
    pub gross_salary: i64,
    pub bpjs_kes_employee: i64,
    pub bpjs_jht_employee: i64,
    pub bpjs_jp_employee: i64,
    pub pph_21_withholding: i64,
    pub other_deductions_json: Option<serde_json::Value>,
    pub total_deductions: i64,
    pub net_salary: i64,
    pub status: String,
    pub employee_name: Option<String>,
    pub department: Option<String>,
    pub exported_at: Option<DateTime<Utc>>,
    pub export_notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub locked_by: Option<Uuid>,
    pub locked_at: Option<DateTime<Utc>>,
    pub created_by_user_nickname: Option<String>,
    pub locked_by_user_nickname: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PayslipLineRecord {
    pub id: Uuid,
    pub payslip_id: Uuid,
    pub work_date: NaiveDate,
    pub days_credited: f32,
    pub ot_hours: f32,
    pub daily_pay: i64,
    pub ot_pay: i64,
    pub holiday_premium: i64,
}

#[derive(Debug, Serialize)]
pub struct PayslipDocument {
    #[serde(flatten)]
    pub payslip: PayslipHeader,
    pub lines: Vec<PayslipLineRecord>,
}

/// One stored attendance row: presence marker plus raw OT hours (converted to minutes for rules).
/// Stored `daily_pay` / `ot_pay` / `holiday_premium` are ignored on
/// purpose: rules-only generate recomputes pay from formulas.
#[derive(Debug, FromRow)]
struct AttendanceLine {
    work_date: NaiveDate,
    ot_hours: Option<f64>,
    is_paid_leave: Option<bool>,
    punch_in: Option<DateTime<Utc>>,
    punch_out: Option<DateTime<Utc>>,
    leave_type: Option<String>,
}

#[derive(Debug)]
struct PayslipAmounts {
    regular_days_pay: i64,
    total_deductions: i64,
    net_salary: i64,
}

/// Zero-filled legacy columns. Rules-only generate stores every rule add
/// in `regular_days_pay` and every rule deduct in `total_deductions`;
/// the old `ot_pay` / `holiday_premium_pay` / `paid_leave_pay` / BPJS /
/// PPh 21 columns stay 0 so the schema is untouched.
fn to_payslip_amounts(gross_adds: f64, total_deducts: f64) -> Option<PayslipAmounts> {
    if !gross_adds.is_finite() || !total_deducts.is_finite() {
        return None;
    }
    if gross_adds < 0.0 || total_deducts < 0.0 {
        return None;
    }
    let gross = gross_adds.round();
    let deducts = total_deducts.round();
    if gross > i64::MAX as f64 || deducts > i64::MAX as f64 {
        return None;
    }
    let gross_salary = gross as i64;
    let total = deducts as i64;
    if total > gross_salary {
        return None;
    }
    Some(PayslipAmounts {
        regular_days_pay: gross_salary,
        total_deductions: total,
        net_salary: gross_salary - total,
    })
}

fn invalid(message: &str) -> HttpResponse {
    actix_web::HttpResponse::BadRequest().json(ApiError::bad_request(message))
}

fn database_error(error: sqlx::Error) -> HttpResponse {
    log::error!("Payroll payslip database operation failed: {error:?}");
    NoDataResponse::new("Database error".to_string(), 500).into_response()
}

async fn require_payroll_access(
    pool: &PgPool,
    bearer: Option<ReqData<String>>,
    action: Action,
) -> Result<String, HttpResponse> {
    let actor = get_session_user(pool.clone(), bearer.clone())
        .await
        .ok_or_else(|| {
            NoDataResponse::new("Session token invalid".to_string(), 401).into_response()
        })?;
    if let Ok(response) = check_action_access(pool.clone(), bearer, "payroll", action).await {
        return Err(response);
    }
    Ok(actor)
}

async fn actor_nickname(pool: &PgPool, nickname: &str) -> Result<String, HttpResponse> {
    sqlx::query_scalar::<_, String>("SELECT user_nickname FROM users WHERE user_nickname = $1")
        .bind(nickname)
        .fetch_optional(pool)
        .await
        .map_err(database_error)?
        .ok_or_else(|| {
            NoDataResponse::new("Session user not found".to_string(), 401).into_response()
        })
}

/// Joined payslip select: payslip row plus employee full name and the
/// salary-row department snapshot. Static literals only (sqlx 0.9 rejects
/// dynamic strings): one for single-row load, one for the filtered list.
const PAYSLIP_SELECT_WHERE_ID: &str = r#"SELECT p.*, TRIM(e.employee_fname || ' ' || e.employee_lname) AS employee_name,
           e.employee_department AS department
    FROM payslips p
    LEFT JOIN employee e ON e.employee_id = p.employee_id
    WHERE p.id = $1"#;

/// Joined payslip list select with optional period window. Static SQL
/// only (sqlx 0.9 rejects dynamic strings); NULL bounds mean unbounded.
const PAYSLIP_LIST_SELECT: &str = r#"SELECT p.*, TRIM(e.employee_fname || ' ' || e.employee_lname) AS employee_name,
           e.employee_department AS department
    FROM payslips p
    LEFT JOIN employee e ON e.employee_id = p.employee_id
    WHERE ($1::DATE IS NULL OR p.period_end >= $1)
      AND ($2::DATE IS NULL OR p.period_start <= $2)
    ORDER BY p.period_start DESC, p.employee_id ASC, p.id ASC LIMIT 500"#;

const PAYSLIP_LIST_PAGED: &str = r#"SELECT p.*, TRIM(e.employee_fname || ' ' || e.employee_lname) AS employee_name,
           e.employee_department AS department
    FROM payslips p
    LEFT JOIN employee e ON e.employee_id = p.employee_id
    WHERE ($1::DATE IS NULL OR p.period_end >= $1)
      AND ($2::DATE IS NULL OR p.period_start <= $2)
    ORDER BY p.period_start DESC, p.employee_id ASC, p.id ASC LIMIT $3 OFFSET $4"#;

const PAYSLIP_COUNT: &str = r#"SELECT count(*)
    FROM payslips p
    WHERE ($1::DATE IS NULL OR p.period_end >= $1)
      AND ($2::DATE IS NULL OR p.period_start <= $2)"#;

async fn load_payslip(
    pool: &PgPool,
    payslip_id: Uuid,
) -> Result<Option<PayslipDocument>, sqlx::Error> {
    let Some(payslip) = sqlx::query_as::<_, PayslipHeader>(PAYSLIP_SELECT_WHERE_ID)
        .bind(payslip_id)
        .fetch_optional(pool)
        .await?
    else {
        return Ok(None);
    };
    let lines = sqlx::query_as::<_, PayslipLineRecord>(
        "SELECT * FROM payslip_lines WHERE payslip_id = $1 ORDER BY work_date, id",
    )
    .bind(payslip_id)
    .fetch_all(pool)
    .await?;
    Ok(Some(PayslipDocument { payslip, lines }))
}

async fn load_payslip_tx(
    tx: &mut Transaction<'_, Postgres>,
    payslip_id: Uuid,
) -> Result<Option<PayslipDocument>, sqlx::Error> {
    let Some(payslip) = sqlx::query_as::<_, PayslipHeader>(PAYSLIP_SELECT_WHERE_ID)
        .bind(payslip_id)
        .fetch_optional(&mut **tx)
        .await?
    else {
        return Ok(None);
    };
    let lines = sqlx::query_as::<_, PayslipLineRecord>(
        "SELECT * FROM payslip_lines WHERE payslip_id = $1 ORDER BY work_date, id",
    )
    .bind(payslip_id)
    .fetch_all(&mut **tx)
    .await?;
    Ok(Some(PayslipDocument { payslip, lines }))
}

#[post("/payroll/payslips/generate")]
pub async fn generate_payslips(
    state: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<GeneratePayslipsRequest>,
) -> impl Responder {
    let actor = match require_payroll_access(&state.db, bearer, Action::Add).await {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    if let Err(error) =
        crate::handler::payroll_payslip::validate_month_period(body.period_start, body.period_end)
    {
        return invalid(error);
    }
    let actor_nickname = match actor_nickname(&state.db, &actor).await {
        Ok(nickname) => nickname,
        Err(response) => return response,
    };
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(error) => return database_error(error),
    };
    if let Err(error) = sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1)::BIGINT)")
        .bind(body.period_start.to_string())
        .execute(&mut *tx)
        .await
    {
        return database_error(error);
    }
    let duplicate_period = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM payslips p LEFT JOIN employee e ON e.employee_id = p.employee_id WHERE p.period_start = $1 AND p.period_end = $2 AND ($3::int IS NULL OR p.employee_id = $3) AND ($4::text IS NULL OR e.employee_department = $4))",
    )
    .bind(body.period_start)
    .bind(body.period_end)
    .bind(body.employee_id)
    .bind(body.department.as_deref())
    .fetch_one(&mut *tx)
    .await
    {
        Ok(exists) => exists,
        Err(error) => return database_error(error),
    };
    if duplicate_period {
        return NoDataResponse::coded(
            "payroll.payslipsExistPeriod",
            "Payslips already exist for this period".to_string(),
            409,
        )
        .into_response();
    }
    // Sync enrolls -> attendance_daily for the period (first punch = in, last = out,
    // overtime = raw seconds out past shift end, no tolerance cut).
    // Idempotent: one row per (employee_id, work_date) via ON CONFLICT, so
    // re-generate never duplicates. Device day uses Asia/Jakarta wall date.
    if let Err(error) = sqlx::query(
        r#"INSERT INTO attendance_daily (employee_id, work_date, punch_in, punch_out, ot_hours)
           SELECT agg.employee_id, agg.wib_date, agg.in_t, agg.out_t,
                  CASE WHEN agg.out_t IS NULL OR agg.shift_end IS NULL THEN NULL
                       WHEN EXTRACT(EPOCH FROM (agg.out_t AT TIME ZONE 'Asia/Jakarta'
                             - (agg.wib_date + (agg.shift_end || ' seconds')::interval)))::bigint
                            > 0 THEN
                         EXTRACT(EPOCH FROM (agg.out_t AT TIME ZONE 'Asia/Jakarta'
                             - (agg.wib_date + (agg.shift_end || ' seconds')::interval)))
                          / 3600.0
                       ELSE 0 END
           FROM (SELECT base.*,
                        CASE WHEN base.cnt > 1 OR base.wall_first < COALESCE(base.shift_start, 28800) + 14400
                             THEN base.first_t END AS in_t,
                        CASE WHEN base.cnt > 1 THEN base.last_t
                             WHEN base.cnt = 1
                              AND base.wall_first >= COALESCE(base.shift_start, 28800) + 14400
                             THEN base.first_t END AS out_t
                 FROM (SELECT emp.employee_id,
                              (to_timestamp(e.enroll_time) AT TIME ZONE 'Asia/Jakarta')::date AS wib_date,
                              COUNT(*) AS cnt,
                              to_timestamp(MIN(e.enroll_time)) AS first_t,
                              to_timestamp(MAX(e.enroll_time)) AS last_t,
                              (EXTRACT(EPOCH FROM (to_timestamp(MIN(e.enroll_time)) AT TIME ZONE 'Asia/Jakarta'))::bigint % 86400) AS wall_first,
                              MAX(sh2.shift_start_time) AS shift_start,
                              MAX(sh2.shift_end_time) AS shift_end
                       FROM enrolls e
                       JOIN employee emp ON emp.employee_device_uid = e.enroll_employee_id
                       JOIN schedules base2 ON base2.schedule_id = emp.employee_schedule_id
                       LEFT JOIN schedules s2 ON base2.schedule_type >= 7
                            AND s2.schedule_parent = base2.schedule_parent
                            AND s2.schedule_dom = EXTRACT(ISODOW FROM (to_timestamp(e.enroll_time) AT TIME ZONE 'Asia/Jakarta'))::int
                       LEFT JOIN shifts sh2 ON sh2.shift_id = COALESCE(s2.schedule_shift_id, base2.schedule_shift_id)
                       WHERE emp.employee_status = 0
                         AND ($1::int IS NULL OR emp.employee_id = $1)
                         AND ($2::text IS NULL OR emp.employee_department = $2)
                         AND to_timestamp(e.enroll_time) AT TIME ZONE 'Asia/Jakarta'
                             BETWEEN $3::date AND ($4::date + INTERVAL '1 day' - INTERVAL '1 second')
                       GROUP BY emp.employee_id, wib_date) AS base) AS agg
           ON CONFLICT (employee_id, work_date) DO UPDATE
             SET punch_in = EXCLUDED.punch_in,
                 punch_out = EXCLUDED.punch_out,
                 ot_hours = EXCLUDED.ot_hours,
                 updated_at = NOW()"#,
    )
    .bind(body.employee_id)
    .bind(body.department.as_deref())
    .bind(body.period_start)
    .bind(body.period_end)
    .execute(&mut *tx)
    .await
    {
        return database_error(error);
    }
    // Approved field attendance fills only the side the device is missing.
    // Pending/rejected days never reach this query, so a day is not credited
    // before the admin decides. Device punches stay authoritative.
    if let Err(error) = sqlx::query(
        r#"INSERT INTO attendance_daily (employee_id, work_date, punch_in, punch_out)
           SELECT f.employee_id, f.work_date,
                  MIN(p.server_at) FILTER (WHERE p.kind = 'in'),
                  MAX(p.server_at) FILTER (WHERE p.kind = 'out')
           FROM field_attendance f
           JOIN employee emp ON emp.employee_id = f.employee_id
           JOIN field_photo p ON p.attendance_id = f.id
           WHERE f.status = 'approved'
             AND emp.employee_status = 0
             AND ($1::int IS NULL OR emp.employee_id = $1)
             AND ($2::text IS NULL OR emp.employee_department = $2)
             AND f.work_date BETWEEN $3::date AND $4::date
           GROUP BY f.employee_id, f.work_date
           ON CONFLICT (employee_id, work_date) DO UPDATE
             SET punch_in = COALESCE(attendance_daily.punch_in, EXCLUDED.punch_in),
                 punch_out = COALESCE(attendance_daily.punch_out, EXCLUDED.punch_out),
                 updated_at = NOW()"#,
    )
    .bind(body.employee_id)
    .bind(body.department.as_deref())
    .bind(body.period_start)
    .bind(body.period_end)
    .execute(&mut *tx)
    .await
    {
        return database_error(error);
    }

    // Overtime for a field day: the enrolls sync above only knows device
    // punches, so a checkout photo past the shift end left ot_hours NULL and
    // the payslip paid zero OT. Recompute it from the merged punch_out with the
    // same formula and the same positive-only rule the enrolls sync uses.
    if let Err(error) = sqlx::query(
        r#"UPDATE attendance_daily ad
           SET ot_hours = CASE
                   WHEN ad.punch_out IS NULL OR sh.shift_end_time IS NULL THEN ad.ot_hours
                   ELSE GREATEST(0, EXTRACT(EPOCH FROM (
                            (ad.punch_out AT TIME ZONE 'Asia/Jakarta')
                            - (ad.work_date + (sh.shift_end_time || ' seconds')::interval))) / 3600.0)
               END,
               updated_at = NOW()
           FROM field_attendance f
           JOIN employee emp2 ON emp2.employee_id = f.employee_id
           JOIN schedules base2 ON base2.schedule_id = emp2.employee_schedule_id
           LEFT JOIN schedules s2 ON base2.schedule_type >= 7
                AND s2.schedule_parent = base2.schedule_parent
                AND s2.schedule_dom = EXTRACT(ISODOW FROM f.work_date)::int
           LEFT JOIN shifts sh ON sh.shift_id = COALESCE(s2.schedule_shift_id, base2.schedule_shift_id)
           WHERE f.status = 'approved'
             AND ad.employee_id = f.employee_id
             AND ad.work_date = f.work_date
             AND ($1::int IS NULL OR emp2.employee_id = $1)
             AND ($2::text IS NULL OR emp2.employee_department = $2)
             AND f.work_date BETWEEN $3::date AND $4::date"#,
    )
    .bind(body.employee_id)
    .bind(body.department.as_deref())
    .bind(body.period_start)
    .bind(body.period_end)
    .execute(&mut *tx)
    .await
    {
        return database_error(error);
    }

    // Shift start/end per employee for late + overtime calc (one query for all in scope).
    // Night shift handled by comparing punch wall-clock vs shift start.
    // Shift per employee AND weekday. The report already picks the schedule row
    // for the day; payroll used the employee's base row, so a Saturday with a
    // shorter shift paid overtime against the wrong end time.
    #[derive(Debug, sqlx::FromRow)]
    struct EmpShift {
        employee_id: i32,
        dom: i32,
        shift_start_time: Option<i64>,
        shift_end_time: Option<i64>,
    }
    let emp_shifts: Vec<EmpShift> = match sqlx::query_as::<_, EmpShift>(
        r#"SELECT e.employee_id,
                  COALESCE(s2.schedule_dom, 0) AS dom,
                  sh.shift_start_time, sh.shift_end_time
           FROM employee e
           JOIN schedules base ON base.schedule_id = e.employee_schedule_id
           LEFT JOIN schedules s2
                  ON base.schedule_type >= 7
                 AND s2.schedule_parent = base.schedule_parent
           LEFT JOIN shifts sh
                  ON sh.shift_id = COALESCE(s2.schedule_shift_id, base.schedule_shift_id)
           WHERE e.employee_status = 0
             AND ($1::int IS NULL OR e.employee_id = $1)
             AND ($2::text IS NULL OR e.employee_department = $2)"#,
    )
    .bind(body.employee_id)
    .bind(body.department.as_deref())
    .fetch_all(&mut *tx)
    .await
    {
        Ok(rows) => rows,
        Err(error) => return database_error(error),
    };
    use std::collections::HashMap;
    let shift_map: HashMap<(i32, i32), EmpShift> = emp_shifts
        .into_iter()
        .map(|r| ((r.employee_id, r.dom), r))
        .collect();
    // Weekly schedules carry one row per weekday; everything else falls back to
    // the base row stored as dom 0.
    let shift_for = |employee_id: i32, date: chrono::NaiveDate| -> Option<&EmpShift> {
        use chrono::Datelike;
        let dom = date.weekday().number_from_monday() as i32;
        shift_map
            .get(&(employee_id, dom))
            .or_else(|| shift_map.get(&(employee_id, 0)))
    };
    // Mark attendance_daily.is_paid_leave + leave_type for approved leave requests in the period.
    // Paid leave overrides stored attendance and receives daily_salary as daily_pay.
    if let Err(error) = sqlx::query(
        r#"UPDATE attendance_daily
           SET is_paid_leave = TRUE,
               leave_type = lr.leave_kind
           FROM leave_requests lr
           WHERE attendance_daily.employee_id = lr.leave_employee_id
             AND attendance_daily.work_date = lr.leave_date
             AND lr.leave_status = 'approved'
             AND lr.leave_date BETWEEN $1 AND $2"#,
    )
    .bind(body.period_start)
    .bind(body.period_end)
    .execute(&mut *tx)
    .await
    {
        return database_error(error);
    }
    // Skip active employees with no salary effective yet for the period.
    // Fail loud only when nobody has salary; partial generation would
    // otherwise block editing one late salary record with a wall of names.
    // Rules come only from salaries.rules_json, so payroll_config is not consulted.
    // Missing rows are reported back as skipped in the response message.
    let missing_salary: Vec<i32> = match sqlx::query_scalar::<_, i32>(
        r#"SELECT e.employee_id FROM employee e
           WHERE e.employee_status = 0
             AND ($2::int IS NULL OR e.employee_id = $2)
             AND ($3::text IS NULL OR e.employee_department = $3)
             AND NOT EXISTS (
                 SELECT 1 FROM salaries s
                 WHERE s.employee_id = e.employee_id AND s.effective_date <= $1
             )
           ORDER BY e.employee_id"#,
    )
    .bind(body.period_start)
    .bind(body.employee_id)
    .bind(body.department.as_deref())
    .fetch_all(&mut *tx)
    .await
    {
        Ok(rows) => rows,
        Err(error) => return database_error(error),
    };
    let skipped_names: Vec<String> = if !missing_salary.is_empty() {
        match sqlx::query_as::<_, (Option<String>, Option<String>, Option<String>)>(
            r#"SELECT e.employee_fname, e.employee_lname, e.employee_department FROM employee e WHERE e.employee_id = ANY($1)"#,
        )
        .bind(&missing_salary)
        .fetch_all(&mut *tx)
        .await
        {
            Ok(rows) => rows
                .into_iter()
                .map(|(first, last, dept)| {
                    let name = [first.unwrap_or_default(), last.unwrap_or_default()]
                        .iter()
                        .map(|part| part.trim().to_string())
                        .filter(|part| !part.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("**{} - {}**", name, dept.unwrap_or_default())
                })
                .collect(),
            Err(error) => return database_error(error),
        }
    } else {
        Vec::new()
    };
    // Rules-only inputs: latest salary row per active employee, plus raw
    // attendance rows (presence + OT hours). Stored daily_pay/ot_pay/
    // holiday_premium are ignored; pay comes from rules_json formulas.
    #[derive(Debug, FromRow)]
    struct SalaryInput {
        employee_id: i32,
        monthly_rate: i64,
        daily_salary: i64,
        hourly_rate: i64,
        rules_json: serde_json::Value,
    }
    let salary_rows = match sqlx::query_as::<_, SalaryInput>(
        r#"SELECT DISTINCT ON (e.employee_id) e.employee_id, s.monthly_rate, s.daily_salary,
                  s.hourly_rate, s.rules_json
           FROM employee e
           JOIN salaries s ON s.employee_id = e.employee_id AND s.effective_date <= $1
           WHERE e.employee_status = 0
             AND ($2::int IS NULL OR e.employee_id = $2)
             AND ($3::text IS NULL OR e.employee_department = $3)
           ORDER BY e.employee_id, s.effective_date DESC"#,
    )
    .bind(body.period_start)
    .bind(body.employee_id)
    .bind(body.department.as_deref())
    .fetch_all(&mut *tx)
    .await
    {
        Ok(rows) => rows,
        Err(error) => return database_error(error),
    };
    if salary_rows.is_empty() {
        return NoDataResponse::new(
            "No employees have an active salary for this period".to_string(),
            409,
        )
        .into_response();
    }
    // Public holidays in the period that count (settings > counted ON): feeds `hols`.
    // Counted OFF means offday: never paid, excluded here to match the report gate.
    let holidays: Vec<NaiveDate> = match sqlx::query_scalar(
        r#"SELECT holiday_date FROM holiday WHERE holiday_date BETWEEN $1 AND $2 AND COALESCE(holiday_included, FALSE)"#,
    )
    .bind(body.period_start)
    .bind(body.period_end)
    .fetch_all(&mut *tx)
    .await
    {
        Ok(rows) => rows,
        Err(error) => return database_error(error),
    };
    let holiday_set: std::collections::HashSet<NaiveDate> =
        holidays.into_iter().collect();
    let mut created_ids = Vec::with_capacity(salary_rows.len());
    for salary in &salary_rows {
        let rules: Vec<SalaryRule> = match serde_json::from_value(salary.rules_json.clone()) {
            Ok(rules) => rules,
            Err(_) => {
                let _ = tx.rollback().await;
                return invalid(&format!(
                    "Employee {} has invalid salary rules",
                    salary.employee_id
                ));
            }
        };
        let attendance = match sqlx::query_as::<_, AttendanceLine>(
            r#"SELECT work_date, ot_hours::float8 AS ot_hours, is_paid_leave, punch_in, punch_out, leave_type FROM attendance_daily
               WHERE employee_id = $1 AND work_date BETWEEN $2 AND $3"#,
        )
        .bind(salary.employee_id)
        .bind(body.period_start)
        .bind(body.period_end)
        .fetch_all(&mut *tx)
        .await
        {
            Ok(rows) => rows,
            Err(error) => return database_error(error),
        };
        let by_date: std::collections::HashMap<NaiveDate, &AttendanceLine> =
            attendance.iter().map(|line| (line.work_date, line)).collect();
        let mut calc_days = Vec::new();
        let mut cursor = body.period_start;
        while cursor <= body.period_end {
            let stored = by_date.get(&cursor);
            let skip = body
                .daily_conditions
                .as_ref()
                .is_some_and(|conditions| {
                    conditions.iter().any(|c| {
                        c.employee_id == salary.employee_id
                            && c.work_date == cursor
                            && matches!(
                                c.condition.as_str(),
                                "absent" | "cuti" | "izin" | "sakit" |
                                "lupa_absen" | "present"
                            )
                    })
                });
            let paid_leave = stored.is_some_and(|line| line.is_paid_leave.unwrap_or(false));
            // Leave kind from the approved-leave mark above: sick counts as
            // sick, izin/cuti count as permission (cuti included per spec).
            let kind = stored.and_then(|line| line.leave_type.clone()).unwrap_or_default();
            let kind = kind.to_lowercase();
            let is_sick = kind == "sakit";
            let is_permission = kind == "izin" || kind == "cuti";
            // Late minutes: punch_in minus shift start, raw no tolerance.
            // punch_in is timestamptz; compare its wall-clock seconds
            // against shift_start_time (seconds from local midnight).
            // attendance_daily rows are stamped in device tz (+7 default),
            // so add 7h to the UTC clock before taking time-of-day.
            let late_minutes = match (stored.and_then(|line| line.punch_in), shift_for(salary.employee_id, cursor).and_then(|s| s.shift_start_time)) {
                (Some(punch), Some(shift_start)) => {
                    let wall = (punch.timestamp() + 7 * 3600).rem_euclid(86400);
                    let late_sec = wall - shift_start;
                    if late_sec <= 0 {
                        0.0
                    } else {
                        (late_sec / 60) as f64
                    }
                }
                _ => 0.0,
            };
            // Manual override first: present forces the day worked with stored
            // OT, absent-like ones keep skip. Leave marks still feed vars.
            let manual_present = body
                .daily_conditions
                .as_ref()
                .is_some_and(|conditions| {
                    conditions.iter().any(|c| {
                        c.employee_id == salary.employee_id
                            && c.work_date == cursor
                            && c.condition == "present"
                    })
                });
            // Half-day rules (report parity):
            // - single punch (in xor out): 0.5, matching the rules engine contract.
            //   A manual `present` upgrades it to a full day.
            // - in+out but arrival past half shift: 0.5 automatic, no override.
            // - in+out normal: 1.0. No punches: 1.0 only via manual override.
            let has_in = stored.and_then(|line| line.punch_in).is_some();
            let has_out = stored.and_then(|line| line.punch_out).is_some();
            let shift_start = shift_for(salary.employee_id, cursor).and_then(|s| s.shift_start_time);
            let punch_wall: Option<i64> = stored.and_then(|line| line.punch_in).map(|punch| (punch.timestamp() + 7 * 3600).rem_euclid(86400));
            let past_half = matches!((punch_wall, shift_start), (Some(wall), Some(start)) if wall > start + 4 * 3600);
            let (day_weight, force_skip) = if has_in && has_out && past_half {
                (0.5, false)
            } else if has_in && has_out {
                (1.0, false)
            } else if has_in || has_out {
                if manual_present { (1.0, false) } else { (0.5, false) }
            } else if manual_present {
                (1.0, false)
            } else {
                (0.0, false)
            };
            calc_days.push(DayInput {
                date: cursor,
                // Stored ot_hours is fractional hours; rules see minutes.
                ot_hours: stored
                    .and_then(|line| line.ot_hours)
                    .unwrap_or(0.0)
                    .max(0.0)
                    * 60.0,
                late_minutes,
                is_sick,
                is_permission,
                is_holiday: holiday_set.contains(&cursor),
                present: ((stored.is_some() && !paid_leave) || manual_present) && !force_skip,
                day_weight,
                paid_leave,
                skip: (skip && !manual_present) || force_skip,
            });
            cursor = cursor.succ_opt().unwrap_or(body.period_end);
            if cursor == body.period_end && calc_days.last().is_some_and(|d: &DayInput| d.date == body.period_end) {
                break;
            }
        }
        let pay_input = EmployeePayInput {
            rates: SalaryRates {
                monthly_rate: salary.monthly_rate as f64,
                daily_salary: salary.daily_salary as f64,
                hourly_rate: salary.hourly_rate as f64,
            },
            rules,
            days: calc_days,
        };
        let pay = match calc_employee_pay(&pay_input) {
            Ok(pay) => pay,
            Err(message) => {
                let who: (Option<String>, Option<String>, Option<String>) =
                    sqlx::query_as(
                        r#"SELECT e.employee_fname, e.employee_lname, e.employee_department FROM employee e WHERE e.employee_id = $1"#,
                    )
                    .bind(salary.employee_id)
                    .fetch_one(&mut *tx)
                    .await
                    .unwrap_or((None, None, None));
                let _ = tx.rollback().await;
                let name = [who.0.unwrap_or_default(), who.1.unwrap_or_default()]
                    .iter()
                    .map(|part| part.trim().to_string())
                    .filter(|part| !part.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                let msg = message;
                return invalid(&format!("**{} - {}**: {msg}", name, who.2.unwrap_or_default()));
            }
        };
        let Some(amounts) = to_payslip_amounts(pay.gross_adds, pay.total_deducts) else {
            let _ = tx.rollback().await;
            return invalid("Payroll values exceed supported range");
        };
        let slip_id = match sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO payslips (
                   employee_id, period_start, period_end, regular_days_pay, ot_pay,
                   holiday_premium_pay, paid_leave_pay, thr_amount, other_earnings_json,
                   gross_salary, bpjs_kes_employee, bpjs_jht_employee, bpjs_jp_employee,
                   pph_21_withholding, other_deductions_json, total_deductions, net_salary,
                   status, created_by_user_nickname
               ) VALUES ($1, $2, $3, $4, 0, 0, 0, 0, '{}'::jsonb, $4, 0, 0, 0, 0, '{}'::jsonb, $5, $6, 'draft', $7)
               RETURNING id"#,
        )
        .bind(salary.employee_id)
        .bind(body.period_start)
        .bind(body.period_end)
        .bind(amounts.regular_days_pay)
        .bind(amounts.total_deductions)
        .bind(amounts.net_salary)
        .bind(&actor_nickname)
        .fetch_one(&mut *tx)
        .await
        {
            Ok(id) => id,
            Err(error) => return database_error(error),
        };
        // DayPay rows carry the computed daily adds as daily_pay. OT/holiday
        // premium columns stay 0; rules decide what any add means.
        for day in &pay.days {
            // A counted day (weight > 0) is written even when no daily rule added
            // money, otherwise a half day vanishes from the payslip while the
            // report still shows it.
            if day.gross_add <= 0.0 && day.weight <= 0.0 {
                continue;
            }
            if let Err(error) = sqlx::query(
                r#"INSERT INTO payslip_lines
                   (payslip_id, work_date, days_credited, ot_hours, daily_pay, ot_pay, holiday_premium)
                   VALUES ($1, $2, $3, $4, $5, 0, 0)"#,
            )
            .bind(slip_id)
            .bind(day.date)
            .bind(day.weight as f32)
            .bind(day.ot_hours as f32)
            .bind(day.gross_add.round() as i64)
            .execute(&mut *tx)
            .await
            {
                return database_error(error);
            }
        }
        // lupa_absen still records a pending_verification row per skipped day.
        if let Some(conditions) = &body.daily_conditions {
            for matched in conditions.iter().filter(|c| {
                c.employee_id == salary.employee_id && c.condition == "lupa_absen"
            }) {
                if let Err(error) = sqlx::query(
                    "INSERT INTO daily_absences (employee_id, absence_date, absence_reason, status, recorded_by) VALUES ($1, $2, 'lupa_absen', 'pending_verification', $3) ON CONFLICT (employee_id, absence_date) DO NOTHING",
                )
                .bind(salary.employee_id)
                .bind(matched.work_date)
                .bind(&actor_nickname)
                .execute(&mut *tx)
                .await
                {
                    return database_error(error);
                }
            }
            if let Some(other) = conditions.iter().find(|c| {
                c.employee_id == salary.employee_id
                    && !matches!(
                        c.condition.as_str(),
                        "absent" | "cuti" | "izin" | "sakit" | "lupa_absen" | "present"
                    )
            }) {
                let _ = tx.rollback().await;
                return invalid(&format!("Unknown daily condition: {}", other.condition));
            }
        }
        created_ids.push(slip_id);
    }
    let mut created = Vec::with_capacity(created_ids.len());
    for id in created_ids {
        match load_payslip_tx(&mut tx, id).await {
            Ok(Some(payslip)) => created.push(payslip),
            Ok(None) => {
                return NoDataResponse::new("Generated payslip not found".to_string(), 500)
                    .into_response();
            }
            Err(error) => return database_error(error),
        }
    }
    if let Err(error) = tx.commit().await {
        return database_error(error);
    }
    let mut message = "Payslips generated".to_string();
    if !skipped_names.is_empty() {
        message.push_str(&format!(
            " (skipped no salary yet: {})",
            skipped_names.join(", ")
        ));
    }
    HttpResponse::Created().json(GenericResponse::new(
        created,
        message,
        201,
    ))
}

#[get("/payroll/payslips")]
pub async fn list_payslips(
    state: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    if let Err(response) = require_payroll_access(&state.db, bearer, Action::View).await {
        return response;
    }
    // Optional period window; absent bound means unbounded on that side.
    let from = query.get("period_start").and_then(|v| v.parse::<NaiveDate>().ok());
    let to = query.get("period_end").and_then(|v| v.parse::<NaiveDate>().ok());
    let limit = query.get("limit").and_then(|v| v.parse::<i64>().ok());

    if let Some(limit) = limit {
        let limit = limit.clamp(1, 200);
        let offset = query.get("offset").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0).max(0);
        let rows = sqlx::query_as::<_, PayslipHeader>(PAYSLIP_LIST_PAGED)
            .bind(from)
            .bind(to)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.db)
            .await;
        let total = sqlx::query_scalar::<_, i64>(PAYSLIP_COUNT)
            .bind(from)
            .bind(to)
            .fetch_one(&state.db)
            .await;
        return match (rows, total) {
            (Ok(rows), Ok(total)) => GenericResponse::<serde_json::Value>::ok(
                vec![serde_json::json!({ "items": rows, "total": total })],
                "Payslips retrieved".to_string(),
            )
            .into_response(),
            (Err(error), _) | (_, Err(error)) => database_error(error),
        };
    }

    match sqlx::query_as::<_, PayslipHeader>(PAYSLIP_LIST_SELECT)
    .bind(from)
    .bind(to)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => GenericResponse::ok(rows, "Payslips retrieved".to_string()).into_response(),
        Err(error) => database_error(error),
    }
}

#[get("/payroll/payslips/{id}")]
pub async fn get_payslip(
    state: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Err(response) = require_payroll_access(&state.db, bearer, Action::View).await {
        return response;
    }
    match load_payslip(&state.db, id.into_inner()).await {
        Ok(Some(payslip)) => {
            GenericResponse::single(payslip, "Payslip retrieved".to_string()).into_response()
        }
        Ok(None) => NoDataResponse::new("Payslip not found".to_string(), 404).into_response(),
        Err(error) => database_error(error),
    }
}

#[post("/payroll/payslips/{id}/lock")]
pub async fn lock_payslip(
    state: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<Uuid>,
) -> impl Responder {
    let actor = match require_payroll_access(&state.db, bearer, Action::Edit).await {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let actor_nickname = match actor_nickname(&state.db, &actor).await {
        Ok(nickname) => nickname,
        Err(response) => return response,
    };
    let id = id.into_inner();
    match sqlx::query_scalar::<_, Uuid>(
        r#"UPDATE payslips SET status = 'locked', locked_by_user_nickname = $1,
               locked_at = NOW(), updated_at = NOW()
           WHERE id = $2 AND status = 'draft' RETURNING id"#,
    )
    .bind(actor_nickname)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(locked_id)) => match load_payslip(&state.db, locked_id).await {
            Ok(Some(payslip)) => {
                GenericResponse::single(payslip, "Payslip locked".to_string()).into_response()
            }
            Ok(None) => {
                NoDataResponse::new("Payslip not found".to_string(), 404).into_response()
            }
            Err(error) => database_error(error),
        },
        Ok(None) => {
            match sqlx::query_scalar::<_, String>("SELECT status FROM payslips WHERE id = $1")
                .bind(id)
                .fetch_optional(&state.db)
                .await
            {
                Ok(None) => {
                    NoDataResponse::new("Payslip not found".to_string(), 404).into_response()
                }
                Ok(Some(status)) => {
                    NoDataResponse::new(format!("Cannot lock payslip in {status} state"), 409)
                        .into_response()
                }
                Err(error) => database_error(error),
            }
        }
        Err(error) => database_error(error),
    }
}

#[delete("/payroll/payslips/{id}")]
pub async fn delete_payslip(
    state: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<Uuid>,
) -> impl Responder {
    if let Err(response) = require_payroll_access(&state.db, bearer, Action::Delete).await {
        return response;
    }
    let payslip_id = id.into_inner();
    let deleted = delete_draft_payslips(&state.db, std::slice::from_ref(&payslip_id)).await;
    match deleted {
        Ok(ids) if !ids.is_empty() => {
            NoDataResponse::new("Payslip deleted".to_string(), 200).into_response()
        }
        Ok(_) => {
            // Nothing deleted: either the id is gone or the row is locked. Saying
            // "not found" for a locked row sent operators hunting for a payslip
            // the list still shows.
            let status: Option<String> =
                sqlx::query_scalar("SELECT status FROM payslips WHERE id = $1")
                    .bind(payslip_id)
                    .fetch_optional(&state.db)
                    .await
                    .unwrap_or(None);
            match status {
                Some(_) => NoDataResponse::coded(
                    "payroll.lockedDelete",
                    "Locked payslip cannot be deleted".to_string(),
                    409,
                )
                .into_response(),
                None => NoDataResponse::new("Payslip not found".to_string(), 404).into_response(),
            }
        }
        Err(error) => database_error(error),
    }
}

/// Bulk delete draft payslips. Locked rows are skipped, never deleted.
/// Returns the ids actually removed.
async fn delete_draft_payslips(
    pool: &PgPool,
    ids: &[Uuid],
) -> Result<Vec<Uuid>, sqlx::Error> {
    let removed = sqlx::query_scalar::<_, Uuid>(
        "DELETE FROM payslips WHERE id = ANY($1) AND status = 'draft' RETURNING id",
    )
    .bind(ids)
    .fetch_all(pool)
    .await?;
    Ok(removed)
}

#[derive(Debug, Deserialize)]
pub struct DeleteDraftPayslipsRequest {
    pub ids: Vec<Uuid>,
}

#[post("/payroll/payslips/delete-drafts")]
pub async fn delete_draft_payslips_bulk(
    state: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<DeleteDraftPayslipsRequest>,
) -> impl Responder {
    if let Err(response) = require_payroll_access(&state.db, bearer, Action::Delete).await {
        return response;
    }
    if body.ids.is_empty() || body.ids.len() > 200 {
        return invalid("Provide 1 to 200 payslip ids");
    }
    match delete_draft_payslips(&state.db, &body.ids).await {
        Ok(removed) => HttpResponse::Ok().json(GenericResponse::new(
            removed.clone(),
            format!("Deleted {} draft payslip(s)", removed.len()),
            200,
        )),
        Err(error) => database_error(error),
    }
}

#[derive(Debug, Deserialize)]
pub struct ApproveDailyAbsenceRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RejectDailyAbsenceRequest {
    pub notes: Option<String>,
}

#[get("/payroll/daily-absences")]
pub async fn list_daily_absences(
    state: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    if let Err(response) = require_payroll_access(&state.db, bearer, Action::View).await {
        return response;
    }
    // Optional status and employee filters; absent means all.
    let status = query.get("status").cloned();
    let employee = query
        .get("employee_id")
        .and_then(|v| v.trim().parse::<i32>().ok())
        .filter(|id| *id > 0);
    let limit = query.get("limit").and_then(|v| v.parse::<i64>().ok());

    if let Some(limit) = limit {
        let limit = limit.clamp(1, 200);
        let offset = query.get("offset").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0).max(0);
        let rows = sqlx::query_as::<_, DailyAbsence>(
            "SELECT * FROM daily_absences WHERE ($1::TEXT IS NULL OR status = $1) AND ($2::INTEGER IS NULL OR employee_id = $2) ORDER BY absence_date DESC, employee_id ASC, id ASC LIMIT $3 OFFSET $4",
        )
        .bind(&status)
        .bind(employee)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await;
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM daily_absences WHERE ($1::TEXT IS NULL OR status = $1) AND ($2::INTEGER IS NULL OR employee_id = $2)",
        )
        .bind(&status)
        .bind(employee)
        .fetch_one(&state.db)
        .await;
        return match (rows, total) {
            (Ok(rows), Ok(total)) => GenericResponse::<serde_json::Value>::ok(
                vec![serde_json::json!({ "items": rows, "total": total })],
                "Daily absences retrieved".to_string(),
            )
            .into_response(),
            (Err(error), _) | (_, Err(error)) => database_error(error),
        };
    }

    match sqlx::query_as::<_, DailyAbsence>(
        "SELECT * FROM daily_absences WHERE ($1::TEXT IS NULL OR status = $1) AND ($2::INTEGER IS NULL OR employee_id = $2) ORDER BY absence_date DESC, employee_id ASC, id ASC LIMIT 500",
    )
    .bind(status)
    .bind(employee)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => GenericResponse::ok(rows, "Daily absences retrieved".to_string()).into_response(),
        Err(error) => database_error(error),
    }
}

#[post("/payroll/daily-absences/{id}/approve")]
pub async fn approve_daily_absence(
    state: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<Uuid>,
    body: web::Json<ApproveDailyAbsenceRequest>,
) -> impl Responder {
    let actor = match require_payroll_access(&state.db, bearer, Action::Edit).await {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let actor_nickname = match actor_nickname(&state.db, &actor).await {
        Ok(nickname) => nickname,
        Err(response) => return response,
    };
    let id = id.into_inner();
    let notes = body.notes.as_deref().unwrap_or("").to_string();
    match sqlx::query_as::<_, DailyAbsence>(
        r#"UPDATE daily_absences SET status = 'approved', approved_by = $1, notes = $2, updated_at = NOW()
           WHERE id = $3 AND status = 'pending_verification' RETURNING *"#,
    )
    .bind(&actor_nickname)
    .bind(notes)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(absence)) => {
            GenericResponse::single(absence, "Daily absence approved".to_string()).into_response()
        }
        Ok(None) => {
            match sqlx::query_scalar::<_, String>("SELECT status FROM daily_absences WHERE id = $1")
                .bind(id)
                .fetch_optional(&state.db)
                .await
            {
                Ok(None) => {
                    NoDataResponse::new("Daily absence not found".to_string(), 404).into_response()
                }
                Ok(Some(status)) => {
                    NoDataResponse::new(format!("Cannot approve daily absence in {status} state"), 409)
                        .into_response()
                }
                Err(error) => database_error(error),
            }
        }
        Err(error) => database_error(error),
    }
}

#[post("/payroll/daily-absences/{id}/reject")]
pub async fn reject_daily_absence(
    state: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<Uuid>,
    body: web::Json<RejectDailyAbsenceRequest>,
) -> impl Responder {
    let actor = match require_payroll_access(&state.db, bearer, Action::Edit).await {
        Ok(actor) => actor,
        Err(response) => return response,
    };
    let actor_nickname = match actor_nickname(&state.db, &actor).await {
        Ok(nickname) => nickname,
        Err(response) => return response,
    };
    let id = id.into_inner();
    let notes = body.notes.as_deref().unwrap_or("").to_string();
    match sqlx::query_as::<_, DailyAbsence>(
        r#"UPDATE daily_absences SET status = 'rejected', approved_by = $1, notes = $2, updated_at = NOW()
           WHERE id = $3 AND status = 'pending_verification' RETURNING *"#,
    )
    .bind(&actor_nickname)
    .bind(notes)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(absence)) => {
            GenericResponse::single(absence, "Daily absence rejected".to_string()).into_response()
        }
        Ok(None) => {
            match sqlx::query_scalar::<_, String>("SELECT status FROM daily_absences WHERE id = $1")
                .bind(id)
                .fetch_optional(&state.db)
                .await
            {
                Ok(None) => {
                    NoDataResponse::new("Daily absence not found".to_string(), 404).into_response()
                }
                Ok(Some(status)) => {
                    NoDataResponse::new(format!("Cannot reject daily absence in {status} state"), 409)
                        .into_response()
                }
                Err(error) => database_error(error),
            }
        }
        Err(error) => database_error(error),
    }
}

#[cfg(test)]
mod tests {
    use super::to_payslip_amounts;

    #[test]
    fn adds_and_deducts_split_into_gross_and_net() {
        let amounts = to_payslip_amounts(200_000.0, 18_000.0).unwrap();
        assert_eq!(amounts.regular_days_pay, 200_000);
        assert_eq!(amounts.total_deductions, 18_000);
        assert_eq!(amounts.net_salary, 182_000);
    }

    #[test]
    fn rejects_non_finite_or_negative_period_net() {
        assert!(to_payslip_amounts(f64::NAN, 1.0).is_none());
        assert!(to_payslip_amounts(200_000.0, f64::INFINITY).is_none());
        // Deducts above adds would net negative.
        assert!(to_payslip_amounts(200_000.0, 200_001.0).is_none());
    }

    #[test]
    fn rejects_negative_amount_inputs() {
        assert!(to_payslip_amounts(-1.0, 0.0).is_none());
        assert!(to_payslip_amounts(200_000.0, -1.0).is_none());
    }
}
