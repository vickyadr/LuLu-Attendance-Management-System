use actix_web::{web, get, post, put, delete, Responder};

use crate::models::payroll::*;
use crate::handler::payroll_expr::{evaluate, required_vars, KNOWN_VARS};
use crate::utility::{
    authorization::{check_action_access, Action},
    stor::{AppState, GenericResponse, IntoResponse, NoDataResponse},
};
use actix_web::web::ReqData;

/// Validate ot_tiers_json: non-empty array of {up_to_hours: number>0|null, multiplier: 1.0-4.0},
/// sorted ascending, only last may be null (open-ended tier).
fn validate_ot_tiers(val: &serde_json::Value) -> Option<actix_web::HttpResponse> {
    use crate::utility::stor::IntoResponse;
    let arr = match val.as_array() {
        Some(a) if !a.is_empty() => a,
        _ => return Some(NoDataResponse::coded("payroll.errOtTiersEmpty", "ot_tiers_json must be a non-empty array".to_string(), 400).into_response()),
    };
    let mut prev: Option<f64> = None;
    for (i, tier) in arr.iter().enumerate() {
        let is_last = i == arr.len() - 1;
        let up_to = tier.get("up_to_hours");
        match up_to {
            Some(v) if v.is_null() => {
                if !is_last {
                    return Some(NoDataResponse::coded("payroll.errOtTiersNullLast", "ot_tiers_json: only last tier may have null up_to_hours".to_string(), 400).into_response());
                }
            }
            Some(v) => {
                let h = match v.as_f64() {
                    Some(h) if h > 0.0 => h,
                    _ => return Some(NoDataResponse::coded("payroll.errOtTiersUpToPositive", "ot_tiers_json: up_to_hours must be > 0 or null".to_string(), 400).into_response()),
                };
                if let Some(p) = prev {
                    if h <= p {
                        return Some(NoDataResponse::coded("payroll.errOtTiersAscending", "ot_tiers_json: up_to_hours must be ascending".to_string(), 400).into_response());
                    }
                }
                prev = Some(h);
            }
            None => return Some(NoDataResponse::coded("payroll.errOtTiersUpToRequired", "ot_tiers_json: each tier needs up_to_hours".to_string(), 400).into_response()),
        }
        let mult = tier.get("multiplier").and_then(|m| m.as_f64()).unwrap_or(0.0);
        if !(1.0..=4.0).contains(&mult) {
            return Some(NoDataResponse::coded("payroll.errOtTiersMultiplier", "ot_tiers_json: multiplier must be 1.0-4.0".to_string(), 400).into_response());
        }
    }
    None
}

// GET /api/payroll/config
#[get("/payroll/config")]
pub async fn get_config(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::View).await {
        return resp;
    }

    match sqlx::query_as::<_, PayrollConfig>(
        r#"SELECT id, work_week_days, daily_work_hours, ot_daily_cap_hours, ot_weekly_cap_hours,
                  ot_rate_first_hour::REAL AS ot_rate_first_hour,
                  ot_rate_subsequent::REAL AS ot_rate_subsequent,
                  rest_day_multiplier::REAL AS rest_day_multiplier,
                  ot_day_threshold_hours, ot_below_mode, ot_tiers_json,
                  holiday_multiplier::REAL AS holiday_multiplier,
                  holiday_basis, carry_unapproved,
                  bpjs_jkk_rate_min::REAL AS bpjs_jkk_rate_min,
                  bpjs_jkk_rate_max::REAL AS bpjs_jkk_rate_max,
                  bpjs_jkm_rate::REAL AS bpjs_jkm_rate,
                  bpjs_jht_employer::REAL AS bpjs_jht_employer,
                  bpjs_jht_employee::REAL AS bpjs_jht_employee,
                  bpjs_jp_employer::REAL AS bpjs_jp_employer,
                  bpjs_jp_employee::REAL AS bpjs_jp_employee,
                  bpjs_kes_employer::REAL AS bpjs_kes_employer,
                  bpjs_kes_employee::REAL AS bpjs_kes_employee,
                  bpjs_kes_wage_cap, meal_ot_threshold_hours, meal_calories, updated_at
           FROM payroll_config LIMIT 1"#
    )
    .fetch_optional(&pool.db)
    .await
    {
        Ok(Some(config)) => GenericResponse::ok(vec![config], "Config retrieved".to_string()).into_response(),
        Ok(None) => NoDataResponse::coded("payroll.errConfigNotFound", "No config found".to_string(), 404).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// PUT /api/payroll/config
#[put("/payroll/config")]
pub async fn update_config(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<UpdatePayrollConfigRequest>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Edit).await {
        return resp;
    }

    // ponytail: Basic field updates; upgrade to transaction + audit log when config changes are tracked
    let mut builder = sqlx::query_builder::QueryBuilder::new(
        "UPDATE payroll_config SET "
    );
    let mut first = true;

    if let Some(val) = body.work_week_days {
        if !first { builder.push(", "); }
        builder.push("work_week_days = ").push_bind(val);
        first = false;
    }
    if let Some(val) = body.daily_work_hours {
        if !first { builder.push(", "); }
        builder.push("daily_work_hours = ").push_bind(val);
        first = false;
    }
    if let Some(val) = body.rest_day_multiplier {
        if !first { builder.push(", "); }
        builder.push("rest_day_multiplier = ").push_bind(val);
        first = false;
    }
    if let Some(val) = body.meal_ot_threshold_hours {
        if !first { builder.push(", "); }
        builder.push("meal_ot_threshold_hours = ").push_bind(val);
        first = false;
    }
    // ponytail: config enums validated in handler, CHECK constraints backstop in DB
    if let Some(val) = body.ot_day_threshold_hours {
        if !(1..=24).contains(&val) {
            return NoDataResponse::coded("payroll.errOtDayThreshold", "ot_day_threshold_hours must be 1-24".to_string(), 400).into_response();
        }
        if !first { builder.push(", "); }
        builder.push("ot_day_threshold_hours = ").push_bind(val);
        first = false;
    }
    if let Some(ref val) = body.ot_below_mode {
        let v = val.to_lowercase();
        if v != "flat" && v != "tiered" {
            return NoDataResponse::coded("payroll.errOtBelowMode", "ot_below_mode must be flat|tiered".to_string(), 400).into_response();
        }
        if !first { builder.push(", "); }
        builder.push("ot_below_mode = ").push_bind(v);
        first = false;
    }
    if let Some(ref val) = body.ot_tiers_json {
        if let Some(resp) = validate_ot_tiers(val) {
            return resp;
        }
        if !first { builder.push(", "); }
        builder.push("ot_tiers_json = ").push_bind(val.clone());
        first = false;
    }
    if let Some(val) = body.holiday_multiplier {
        if !(1.0..=4.0).contains(&val) {
            return NoDataResponse::coded("payroll.errHolidayMultiplier", "holiday_multiplier must be 1.0-4.0".to_string(), 400).into_response();
        }
        if !first { builder.push(", "); }
        builder.push("holiday_multiplier = ").push_bind(val);
        first = false;
    }
    if let Some(ref val) = body.holiday_basis {
        let v = val.to_lowercase();
        if v != "hourly" && v != "daily" {
            return NoDataResponse::coded("payroll.errHolidayBasis", "holiday_basis must be hourly|daily".to_string(), 400).into_response();
        }
        if !first { builder.push(", "); }
        builder.push("holiday_basis = ").push_bind(v);
        first = false;
    }
    if let Some(ref val) = body.carry_unapproved {
        let v = val.to_lowercase();
        if v != "never" && v != "next" && v != "backpay" {
            return NoDataResponse::coded("payroll.errCarryUnapproved", "carry_unapproved must be never|next|backpay".to_string(), 400).into_response();
        }
        if !first { builder.push(", "); }
        builder.push("carry_unapproved = ").push_bind(v);
        first = false;
    }

    if first {
        return NoDataResponse::coded("payroll.errNoFields", "No fields to update".to_string(), 400).into_response();
    }

    builder.push(
        r#" WHERE id = (SELECT id FROM payroll_config LIMIT 1)
            RETURNING id, work_week_days, daily_work_hours, ot_daily_cap_hours, ot_weekly_cap_hours,
                      ot_rate_first_hour::REAL AS ot_rate_first_hour,
                      ot_rate_subsequent::REAL AS ot_rate_subsequent,
                      rest_day_multiplier::REAL AS rest_day_multiplier,
                      ot_day_threshold_hours, ot_below_mode, ot_tiers_json,
                      holiday_multiplier::REAL AS holiday_multiplier,
                      holiday_basis, carry_unapproved,
                      bpjs_jkk_rate_min::REAL AS bpjs_jkk_rate_min,
                      bpjs_jkk_rate_max::REAL AS bpjs_jkk_rate_max,
                      bpjs_jkm_rate::REAL AS bpjs_jkm_rate,
                      bpjs_jht_employer::REAL AS bpjs_jht_employer,
                      bpjs_jht_employee::REAL AS bpjs_jht_employee,
                      bpjs_jp_employer::REAL AS bpjs_jp_employer,
                      bpjs_jp_employee::REAL AS bpjs_jp_employee,
                      bpjs_kes_employer::REAL AS bpjs_kes_employer,
                      bpjs_kes_employee::REAL AS bpjs_kes_employee,
                      bpjs_kes_wage_cap, meal_ot_threshold_hours, meal_calories, updated_at"#,
    );

    match builder.build_query_as::<PayrollConfig>()
        .fetch_one(&pool.db)
        .await
    {
        Ok(config) => GenericResponse::ok(vec![config], "Config updated".to_string()).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// POST /api/payroll/salary-templates
#[post("/payroll/salary-templates")]
pub async fn create_salary_template(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<CreateSalaryTemplateRequest>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Add).await {
        return resp;
    }

    match sqlx::query_as::<_, SalaryTemplate>(
        r#"INSERT INTO salary_templates 
           (template_name, description, basic_salary_type, hra_percent, hra_fixed, transport_fixed, monthly_rate, daily_rate, hourly_rate, rules_json, ot_eligible)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           RETURNING *"#
    )
    .bind(&body.template_name)
    .bind(&body.description)
    .bind(&body.basic_salary_type)
    .bind(body.hra_percent)
    .bind(body.hra_fixed)
    .bind(body.transport_fixed)
    .bind(body.monthly_rate)
    .bind(body.daily_rate)
    .bind(body.hourly_rate)
    .bind(body.rules_json.clone().unwrap_or(serde_json::json!([])))
    .bind(body.ot_eligible.unwrap_or(true))
    .fetch_one(&pool.db)
    .await
    {
        Ok(template) => GenericResponse::ok(vec![template], "Template created".to_string()).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// GET /api/payroll/salary-templates
#[get("/payroll/salary-templates")]
pub async fn list_salary_templates(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::View).await {
        return resp;
    }

    match sqlx::query_as::<_, SalaryTemplate>(
        "SELECT * FROM salary_templates ORDER BY template_name ASC"
    )
    .fetch_all(&pool.db)
    .await
    {
        Ok(templates) => GenericResponse::ok(templates, "Templates retrieved".to_string()).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// GET /api/payroll/salary-templates/:id
#[get("/payroll/salary-templates/{id}")]
pub async fn get_salary_template(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<uuid::Uuid>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::View).await {
        return resp;
    }

    let template_id = id.into_inner();
    
    match sqlx::query_as::<_, SalaryTemplate>(
        "SELECT * FROM salary_templates WHERE id = $1"
    )
    .bind(&template_id)
    .fetch_optional(&pool.db)
    .await
    {
        Ok(Some(template)) => GenericResponse::ok(vec![template], "Template retrieved".to_string()).into_response(),
        Ok(None) => NoDataResponse::coded("payroll.errTemplateNotFound", "Template not found".to_string(), 404).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// PUT /api/payroll/salary-templates/:id
#[put("/payroll/salary-templates/{id}")]
pub async fn update_salary_template(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<uuid::Uuid>,
    body: web::Json<UpdateSalaryTemplateRequest>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Edit).await {
        return resp;
    }

    let template_id = id.into_inner();

    // Optional: validate new rules before storing on template or linked salaries.
    let template_rules: Option<serde_json::Value> = match &body.rules_json {
        None => None,
        Some(raw) => match validate_salary_rules(raw) {
            Ok(normalized) => Some(serde_json::to_value(&normalized).unwrap_or(serde_json::json!([]))),
            Err(msg) => return NoDataResponse::new(msg, 400).into_response(),
        },
    };

    let mut tx = match pool.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("template {}: begin failed: {}", template_id, e);
            return NoDataResponse::new("Database error".to_string(), 500).into_response();
        }
    };

    let template = match sqlx::query_as::<_, SalaryTemplate>(
        r#"UPDATE salary_templates
           SET description = COALESCE($1, description),
               hra_percent = COALESCE($2, hra_percent),
               hra_fixed = COALESCE($3, hra_fixed),
               transport_fixed = COALESCE($4, transport_fixed),
               monthly_rate = COALESCE($5, monthly_rate),
               daily_rate = COALESCE($6, daily_rate),
               hourly_rate = COALESCE($7, hourly_rate),
               rules_json = COALESCE($8, rules_json),
               ot_eligible = COALESCE($9, ot_eligible),
               updated_at = NOW()
           WHERE id = $10
           RETURNING *"#,
    )
    .bind(&body.description)
    .bind(body.hra_percent)
    .bind(body.hra_fixed)
    .bind(body.transport_fixed)
    .bind(body.monthly_rate)
    .bind(body.daily_rate)
    .bind(body.hourly_rate)
    .bind(&template_rules)
    .bind(body.ot_eligible)
    .bind(&template_id)
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(template)) => template,
        Ok(None) => return NoDataResponse::coded("payroll.errTemplateNotFound", "Template not found".to_string(), 404).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            return NoDataResponse::new("Database error".to_string(), 500).into_response();
        }
    };

    // Opsi A: rules live. Linked salaries follow template rules; amounts untouched.
    // Same transaction as the template row, so a failure cannot leave the two out
    // of sync while the response says success.
    if template_rules.is_some() {
        if let Err(e) = sqlx::query(
            r#"UPDATE salaries SET rules_json = $1
               WHERE id IN (SELECT DISTINCT ON (employee_id) id FROM salaries
                            WHERE template_id = $2
                            ORDER BY employee_id, effective_date DESC)"#,
        )
        .bind(&template.rules_json)
        .bind(&template_id)
        .execute(&mut *tx)
        .await
        {
            log::error!("template {} rule propagation failed: {}", template_id, e);
            return NoDataResponse::new("Database error".to_string(), 500).into_response();
        }
    }

    if let Err(e) = tx.commit().await {
        log::error!("template {}: commit failed: {}", template_id, e);
        return NoDataResponse::new("Database error".to_string(), 500).into_response();
    }

    GenericResponse::ok(vec![template], "Template updated".to_string()).into_response()
}

fn unescape_csv_export(text: &str) -> &str {
    match text.strip_prefix('\'') {
        Some(rest) if rest.starts_with(['=', '+', '-', '@']) => rest,
        _ => text,
    }
}

fn normalize_salary_rule(raw: &serde_json::Value) -> Result<SalaryRule, String> {
    // New shape: the formula lives in `function`, or in `parameter` when the
    // caller only sends the formula field. Both must skip the legacy branch -
    // otherwise a rule with a formula but no `function` was rewritten to `0`.
    if raw.get("function").is_some() || raw.get("parameter").is_some() {
        let mut value = raw.clone();
        if value.get("function").is_none() {
            if let Some(parameter) = value.get("parameter").cloned() {
                value["function"] = parameter;
            }
        }
        return serde_json::from_value(value).map_err(|_| {
            "need name, function, add, deduct, daily".to_string()
        });
    }
    let name = raw
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let kind = raw.get("kind").and_then(|v| v.as_str()).unwrap_or("add");
    let basis = raw
        .get("basis")
        .and_then(|v| v.as_str())
        .unwrap_or("fixed");
    let amount = raw
        .get("amount")
        .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0);
    let multiplier = raw
        .get("multiplier")
        .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(1.0);
    let mult = if multiplier == 1.0 {
        String::new()
    } else {
        format!(" * {multiplier}")
    };
    let function = match basis {
        "absent" => format!("absent * {amount}{mult}"),
        "present" => format!("present * {amount}{mult}"),
        "overtime" => format!("overtime * {amount} / 60{mult}"),
        _ => format!("{amount}{mult}"),
    };
    Ok(SalaryRule {
        name,
        function,
        add: kind != "deduct",
        deduct: kind == "deduct",
        parameter: None,
        daily: basis != "fixed",
    })
}

/// Parse the CSV effective date. Excel writes `11/01/26` for a date typed as
/// `11/01/2026`, and can hand back `11-01-26`, so accept every separator
/// combination with two- or four-digit years. ISO `YYYY-MM-DD` is accepted
/// too (BE export shape).
fn parse_effective_date(raw: &str) -> Option<chrono::NaiveDate> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let normalized = trimmed.replace('-', "/");
    let parts: Vec<&str> = normalized.split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    // chrono's %y maps "26" to year 0026, so expand two-digit years by hand.
    let (year, month, day) = if parts[0].len() == 4 {
        let y = parts[0].parse::<i32>().ok()?;
        (y, parts[1].parse::<u32>().ok()?, parts[2].parse::<u32>().ok()?)
    } else {
        let mut y = parts[2].parse::<i32>().ok()?;
        if parts[2].len() <= 2 {
            y += if y < 70 { 2000 } else { 1900 };
        }
        (y, parts[1].parse::<u32>().ok()?, parts[0].parse::<u32>().ok()?)
    };
    chrono::NaiveDate::from_ymd_opt(year, month, day)
}

#[cfg(test)]
mod date_tests {
    use super::parse_effective_date;

    fn day(text: &str) -> String {
        parse_effective_date(text)
            .map(|d| d.to_string())
            .unwrap_or_else(|| "invalid".to_string())
    }

    #[test]
    fn parses_the_formats_the_readme_promises() {
        // Excel round-trip, four- and two-digit years, dashes, ISO.
        assert_eq!(day("11/01/2026"), "2026-01-11");
        assert_eq!(day("11/01/26"), "2026-01-11");
        assert_eq!(day("11-01-2026"), "2026-01-11");
        assert_eq!(day("11-01-26"), "2026-01-11");
        assert_eq!(day("2026-01-11"), "2026-01-11");
        assert_eq!(day(" 11/01/2026 "), "2026-01-11");
    }

    #[test]
    fn expands_two_digit_years_instead_of_year_26() {
        assert_eq!(day("01/02/69"), "2069-02-01");
        assert_eq!(day("01/02/70"), "1970-02-01");
    }

    #[test]
    fn validates_the_calendar() {
        assert_eq!(day("29/02/2024"), "2024-02-29");
        assert_eq!(day("29/02/26"), "invalid");
        assert_eq!(day("31/04/2026"), "invalid");
        assert_eq!(day(""), "invalid");
        assert_eq!(day("11/01"), "invalid");
    }
}

/// Validate rules_json: array of SalaryRule {name, function, add, deduct, parameter, daily}.
/// FE 3-col shape sends the kind label in function (Add/Deduct) and the
/// formula in parameter, so validate the effective formula, not the label.
/// Legacy rows carry the formula directly in function.
/// parameter accepts raw formula or `=` prefixed (FE convention);
/// a single optional leading `=` is stripped before var checks.
/// Legacy FE1 shape {name, kind, basis, amount, multiplier} is normalized to a
/// concrete formula so payroll.vue assign works until FE2 ships the formula UI.
fn validate_salary_rules(val: &serde_json::Value) -> Result<Vec<SalaryRule>, String> {
    let arr = val
        .as_array()
        .ok_or_else(|| "rules_json must be an array".to_string())?;
    let mut rules = Vec::with_capacity(arr.len());
    for (i, raw) in arr.iter().enumerate() {
        let rule = normalize_salary_rule(raw)
            .map_err(|e| format!("rules_json[{i}]: {e}"))?;
        if rule.name.trim().is_empty() {
            return Err(format!("rules_json[{i}]: name is empty"));
        }
        if rule.function.trim().is_empty() {
            return Err(format!("rules_json[{i}]: function is empty"));
        }
        if rule.add == rule.deduct {
            return Err(format!("rules_json[{i}]: exactly one of add/deduct must be true"));
        }
        // Raw formula and `=` prefixed both pass; required_vars strips
        // one optional leading `=` before parsing, so no prefix check here.
        let used = required_vars(rule.expression())
            .map_err(|e| format!("rules_json[{i}]: bad function: {e}"))?;
        let unknown: Vec<_> = used
            .iter()
            .filter(|v| !KNOWN_VARS.contains(&v.as_str()))
            .collect();
        if !unknown.is_empty() {
            let names: Vec<&str> = unknown.iter().map(|v| v.as_str()).collect();
            return Err(format!("rules_json[{i}]: unknown vars: {}", names.join(", ")));
        }
        // required_vars only tokenizes; run the parser too so unbalanced
        // parens, stray operators and unknown functions are rejected here
        // instead of at payroll time. Variables are all 1.0, so a divisor that
        // is zero for that input (salary / (present - 1)) is rejected too: the
        // checker cannot tell a parse error from an eval error today.
        let dummy: std::collections::HashMap<String, f64> =
            KNOWN_VARS.iter().map(|v| (v.to_string(), 1.0)).collect();
        evaluate(rule.expression(), &dummy)
            .map_err(|e| format!("rules_json[{i}]: bad formula: {e}"))?;
        rules.push(rule);
    }
    Ok(rules)
}

// POST /api/payroll/employee-salary
#[post("/payroll/employee-salary")]
pub async fn assign_employee_salary(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<CreateEmployeeSalaryRequest>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Add).await {
        return resp;
    }

    if body.salary_type != "monthly" && body.salary_type != "daily" {
        return NoDataResponse::coded("payroll.errSalaryType", "salary_type must be monthly or daily".to_string(), 400).into_response();
    }
    if body.monthly_rate < 0 || body.daily_salary < 0 || body.hourly_rate < 0 {
        return NoDataResponse::coded("payroll.errSalaryAmountNonNegative", "salary amounts must be >= 0".to_string(), 400).into_response();
    }
    // Opsi A: assign with template_id links the live template rules; amounts stay
    // per-salary from the request. Empty/missing rules + template = copy template.
    let mut rules_value = body.rules_json.clone().unwrap_or(serde_json::json!([]));
    let rules_empty = rules_value.as_array().is_some_and(|a| a.is_empty());
    if rules_empty {
        if let Some(tid) = &body.template_id {
            if let Ok(Some(tpl)) = sqlx::query_as::<_, SalaryTemplate>(
                "SELECT * FROM salary_templates WHERE id = $1",
            )
            .bind(tid)
            .fetch_optional(&pool.db)
            .await
            {
                rules_value = tpl.rules_json;
            }
        }
    }
    match validate_salary_rules(&rules_value) {
        Ok(normalized) => {
            rules_value =
                serde_json::to_value(&normalized).unwrap_or(serde_json::json!([]));
        }
        Err(msg) => {
            return NoDataResponse::new(msg, 400).into_response();
        }
    }

    match sqlx::query_as::<_, EmployeeSalary>(
        r#"INSERT INTO salaries
           (employee_id, template_id, salary_type, monthly_rate, daily_salary, hourly_rate, basic_salary, department, rules_json, ot_eligible, effective_date)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           RETURNING *"#
    )
    .bind(body.employee_id)
    .bind(&body.template_id)
    .bind(&body.salary_type)
    .bind(body.monthly_rate)
    .bind(body.daily_salary)
    .bind(body.hourly_rate)
    .bind(body.basic_salary)
    .bind(body.department.clone())
    .bind(&rules_value)
    .bind(body.ot_eligible.unwrap_or(true))
    .bind(body.effective_date)
    .fetch_one(&pool.db)
    .await
    {
        Ok(salary) => GenericResponse::ok(vec![salary], "Salary assigned".to_string()).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            // Duplicate (employee_id, effective_date): guide user to newer date.
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().as_deref() == Some("23505") {
                    return NoDataResponse::coded(
                        "payroll.salaryDateExists",
                        "Salary for this effective date already exists. Use a newer date or delete the row first.".to_string(),
                        409,
                    )
                    .into_response();
                }
            }
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// PUT /api/payroll/employee-salary/:id/rules — edit rules_json only.
// Amounts, type, dates stay locked: changing them means delete row + add new row.
#[put("/payroll/employee-salary/{id}/rules")]
pub async fn update_salary_rules(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<uuid::Uuid>,
    body: web::Json<UpdateSalaryRulesRequest>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Edit).await {
        return resp;
    }

    let normalized = match validate_salary_rules(&body.rules_json) {
        Ok(r) => serde_json::to_value(&r).unwrap_or(serde_json::json!([])),
        Err(msg) => return NoDataResponse::new(msg, 400).into_response(),
    };

    match sqlx::query_as::<_, EmployeeSalary>(
        "UPDATE salaries SET rules_json = $1 WHERE id = $2 RETURNING *"
    )
    .bind(normalized)
    .bind(id.into_inner())
    .fetch_optional(&pool.db)
    .await
    {
        Ok(Some(row)) => GenericResponse::ok(vec![row], "Salary rules updated".to_string()).into_response(),
        Ok(None) => NoDataResponse::coded("payroll.errSalaryNotFound", "Salary not found".to_string(), 404).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// DELETE /api/payroll/employee-salary/:id — delete salary row (append-only versioning)
#[delete("/payroll/employee-salary/{id}")]
pub async fn delete_employee_salary(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<uuid::Uuid>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Delete).await {
        return resp;
    }

    match sqlx::query("DELETE FROM salaries WHERE id = $1")
    .bind(id.into_inner())
    .execute(&pool.db)
    .await
    {
        Ok(done) if done.rows_affected() > 0 => NoDataResponse::new("Salary deleted".to_string(), 200).into_response(),
        Ok(_) => NoDataResponse::coded("payroll.errSalaryNotFound", "Salary not found".to_string(), 404).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// POST /api/payroll/employee-salary-copy-rules — copy rules_json from one salary row to another
// Flat path (not nested under /employee-salary/) so actix never confuses
// "copy-rules" with the GET /{employee_id} segment.
#[post("/payroll/employee-salary-copy-rules")]
pub async fn copy_salary_rules(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<CopySalaryRulesRequest>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Edit).await {
        return resp;
    }

    if body.from_employee_id == body.to_employee_id {
        return NoDataResponse::coded("payroll.errSameEmployee", "from_employee_id and to_employee_id must differ".to_string(), 400).into_response();
    }

    let src: Option<EmployeeSalary> = match sqlx::query_as::<_, EmployeeSalary>(
        "SELECT * FROM salaries WHERE employee_id = $1 ORDER BY effective_date DESC LIMIT 1",
    )
    .bind(body.from_employee_id)
    .fetch_optional(&pool.db)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            log::error!("DB error: {}", e);
            return NoDataResponse::new("Database error".to_string(), 500).into_response();
        }
    };
    let src = match src {
        Some(row) => row,
        None => return NoDataResponse::coded("payroll.errSourceSalaryNotFound", "Source salary not found".to_string(), 404).into_response(),
    };
    if let Err(msg) = validate_salary_rules(&src.rules_json) {
        return NoDataResponse::new(format!("source rules invalid: {msg}"), 422).into_response();
    }

    match sqlx::query_as::<_, EmployeeSalary>(
        r#"UPDATE salaries SET rules_json = $1, updated_at = NOW()
           WHERE id = (SELECT id FROM salaries WHERE employee_id = $2 ORDER BY effective_date DESC LIMIT 1)
           RETURNING *"#
    )
    .bind(&src.rules_json)
    .bind(body.to_employee_id)
    .fetch_optional(&pool.db)
    .await
    {
        Ok(Some(salary)) => GenericResponse::ok(vec![salary], "Rules copied".to_string()).into_response(),
        Ok(None) => NoDataResponse::coded("payroll.errTargetSalaryNotFound", "Target salary not found".to_string(), 404).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// GET /api/payroll/employee-salaries — all salary rows, newest first
#[get("/payroll/employee-salaries")]
pub async fn list_employee_salaries(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::View).await {
        return resp;
    }

    match sqlx::query_as::<_, EmployeeSalary>(
        "SELECT * FROM salaries ORDER BY employee_id ASC, effective_date DESC"
    )
    .fetch_all(&pool.db)
    .await
    {
        Ok(rows) => GenericResponse::ok(rows, "Salaries retrieved".to_string()).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// GET /api/payroll/employee-salary/:employee_id
#[get("/payroll/employee-salary/{employee_id}")]
pub async fn get_employee_salary(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    employee_id: web::Path<i32>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::View).await {
        return resp;
    }

    match sqlx::query_as::<_, EmployeeSalary>(
        "SELECT * FROM salaries WHERE employee_id = $1 ORDER BY effective_date DESC LIMIT 1"
    )
    .bind(employee_id.into_inner())
    .fetch_optional(&pool.db)
    .await
    {
        Ok(Some(salary)) => GenericResponse::ok(vec![salary], "Salary retrieved".to_string()).into_response(),
        Ok(None) => NoDataResponse::coded("payroll.errSalaryNotFound", "Salary not found".to_string(), 404).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// POST /api/payroll/public-holidays
#[post("/payroll/public-holidays")]
pub async fn create_public_holiday(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<CreatePublicHolidayRequest>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Add).await {
        return resp;
    }

    match sqlx::query_as::<_, PublicHoliday>(
        r#"INSERT INTO public_holidays 
           (holiday_date, holiday_name, holiday_type, ot_rate_multiplier_min, ot_rate_multiplier_max)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING *"#
    )
    .bind(body.holiday_date)
    .bind(&body.holiday_name)
    .bind(&body.holiday_type)
    .bind(body.ot_rate_multiplier_min)
    .bind(body.ot_rate_multiplier_max)
    .fetch_one(&pool.db)
    .await
    {
        Ok(holiday) => GenericResponse::ok(vec![holiday], "Holiday created".to_string()).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// GET /api/payroll/public-holidays
#[get("/payroll/public-holidays")]
pub async fn list_public_holidays(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::View).await {
        return resp;
    }

    match sqlx::query_as::<_, PublicHoliday>(
        "SELECT * FROM public_holidays ORDER BY holiday_date DESC"
    )
    .fetch_all(&pool.db)
    .await
    {
        Ok(holidays) => GenericResponse::ok(holidays, "Holidays retrieved".to_string()).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// DELETE /api/payroll/public-holidays/:id
#[delete("/payroll/public-holidays/{id}")]
pub async fn delete_public_holiday(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<uuid::Uuid>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Delete).await {
        return resp;
    }

    match sqlx::query("DELETE FROM public_holidays WHERE id = $1")
        .bind(id.into_inner())
        .execute(&pool.db)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                NoDataResponse::new("Holiday deleted".to_string(), 200).into_response()
            } else {
                NoDataResponse::new("Holiday not found".to_string(), 404).into_response()
            }
        }
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

const DEDUCTION_KINDS: [&str; 4] = ["permit", "sick", "absent", "late"];
const DEDUCTION_POLICIES: [&str; 4] = ["standard", "strict", "lenient", "custom"];

fn valid_deduction_input(
    kind: &str,
    policy: &str,
    grace_minutes: i32,
    fixed_amount: i64,
    template_name: &str,
) -> Option<actix_web::HttpResponse> {
    use crate::utility::stor::IntoResponse;
    if template_name.trim().is_empty() {
        return Some(NoDataResponse::new("template_name is required".to_string(), 400).into_response());
    }
    if !DEDUCTION_KINDS.contains(&kind) {
        return Some(NoDataResponse::new(
            "Invalid kind (permit | sick | absent | late)".to_string(), 400,
        ).into_response());
    }
    if !DEDUCTION_POLICIES.contains(&policy) {
        return Some(NoDataResponse::new(
            "Invalid policy (standard | strict | lenient | custom)".to_string(), 400,
        ).into_response());
    }
    if grace_minutes < 0 {
        return Some(NoDataResponse::new("grace_minutes must be >= 0".to_string(), 400).into_response());
    }
    if grace_minutes > 0 && kind != "late" {
        return Some(NoDataResponse::new(
            "grace_minutes only applies to kind=late".to_string(), 400,
        ).into_response());
    }
    if fixed_amount < 0 {
        return Some(NoDataResponse::new("fixed_amount must be >= 0".to_string(), 400).into_response());
    }
    None
}

// POST /api/payroll/deduction-templates
#[post("/payroll/deduction-templates")]
pub async fn create_deduction_template(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<CreateDeductionTemplateRequest>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Add).await {
        return resp;
    }

    let policy = body.policy.clone().unwrap_or_else(|| "standard".to_string());
    let grace = body.grace_minutes.unwrap_or(0);
    let amount = body.fixed_amount.unwrap_or(0);
    if let Some(err) = valid_deduction_input(&body.kind, &policy, grace, amount, &body.template_name) {
        return err;
    }

    match sqlx::query_as::<_, DeductionTemplate>(
        r#"INSERT INTO deduction_templates
           (template_name, description, kind, policy, grace_minutes, fixed_amount, is_active)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING *"#
    )
    .bind(body.template_name.trim())
    .bind(&body.description)
    .bind(&body.kind)
    .bind(&policy)
    .bind(grace)
    .bind(amount)
    .bind(body.is_active.unwrap_or(true))
    .fetch_one(&pool.db)
    .await
    {
        Ok(template) => GenericResponse::ok(vec![template], "Deduction template created".to_string()).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    return NoDataResponse::new("template_name already exists".to_string(), 409).into_response();
                }
                if db_err.code().as_deref() == Some("23514") {
                    return NoDataResponse::new("Invalid kind/policy/value rejected by database".to_string(), 400).into_response();
                }
            }
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// GET /api/payroll/deduction-templates
#[get("/payroll/deduction-templates")]
pub async fn list_deduction_templates(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::View).await {
        return resp;
    }

    match sqlx::query_as::<_, DeductionTemplate>(
        "SELECT * FROM deduction_templates ORDER BY template_name ASC"
    )
    .fetch_all(&pool.db)
    .await
    {
        Ok(templates) => GenericResponse::ok(templates, "Deduction templates retrieved".to_string()).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// GET /api/payroll/deduction-templates/:id
#[get("/payroll/deduction-templates/{id}")]
pub async fn get_deduction_template(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<uuid::Uuid>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::View).await {
        return resp;
    }

    match sqlx::query_as::<_, DeductionTemplate>(
        "SELECT * FROM deduction_templates WHERE id = $1"
    )
    .bind(&id.into_inner())
    .fetch_optional(&pool.db)
    .await
    {
        Ok(Some(template)) => GenericResponse::ok(vec![template], "Deduction template retrieved".to_string()).into_response(),
        Ok(None) => NoDataResponse::new("Deduction template not found".to_string(), 404).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// PUT /api/payroll/deduction-templates/:id
#[put("/payroll/deduction-templates/{id}")]
pub async fn update_deduction_template(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<uuid::Uuid>,
    body: web::Json<UpdateDeductionTemplateRequest>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Edit).await {
        return resp;
    }

    let template_id = id.into_inner();
    let current: Option<DeductionTemplate> = sqlx::query_as(
        "SELECT * FROM deduction_templates WHERE id = $1"
    )
    .bind(&template_id)
    .fetch_optional(&pool.db)
    .await
    .unwrap_or(None);
    let Some(cur) = current else {
        return NoDataResponse::new("Deduction template not found".to_string(), 404).into_response();
    };

    let kind = body.kind.clone().unwrap_or(cur.kind);
    let policy = body.policy.clone().unwrap_or(cur.policy);
    let grace = body.grace_minutes.unwrap_or(cur.grace_minutes);
    let amount = body.fixed_amount.unwrap_or(cur.fixed_amount);
    if let Some(err) = valid_deduction_input(&kind, &policy, grace, amount, &cur.template_name) {
        return err;
    }

    match sqlx::query_as::<_, DeductionTemplate>(
        r#"UPDATE deduction_templates
           SET description = COALESCE($1, description),
               kind = $2,
               policy = $3,
               grace_minutes = $4,
               fixed_amount = $5,
               is_active = COALESCE($6, is_active),
               updated_at = NOW()
           WHERE id = $7
           RETURNING *"#
    )
    .bind(&body.description)
    .bind(&kind)
    .bind(&policy)
    .bind(grace)
    .bind(amount)
    .bind(body.is_active)
    .bind(&template_id)
    .fetch_optional(&pool.db)
    .await
    {
        Ok(Some(template)) => GenericResponse::ok(vec![template], "Deduction template updated".to_string()).into_response(),
        Ok(None) => NoDataResponse::new("Deduction template not found".to_string(), 404).into_response(),
        Err(e) => {
            log::error!("DB error: {}", e);
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23514") {
                    return NoDataResponse::new("Invalid kind/policy/value rejected by database".to_string(), 400).into_response();
                }
            }
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// DELETE /api/payroll/deduction-templates/:id
#[delete("/payroll/deduction-templates/{id}")]
pub async fn delete_deduction_template(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    id: web::Path<uuid::Uuid>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Delete).await {
        return resp;
    }

    match sqlx::query("DELETE FROM deduction_templates WHERE id = $1")
        .bind(id.into_inner())
        .execute(&pool.db)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                NoDataResponse::new("Deduction template deleted".to_string(), 200).into_response()
            } else {
                NoDataResponse::new("Deduction template not found".to_string(), 404).into_response()
            }
        }
        Err(e) => {
            log::error!("DB error: {}", e);
            NoDataResponse::new("Database error".to_string(), 500).into_response()
        }
    }
}

// POST /api/payroll/employee-salaries/import
#[post("/payroll/employee-salaries/import")]
pub async fn import_employee_salaries(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<serde_json::Value>,
) -> impl Responder {
    // Replacing an effective-dated salary row is a delete. A clerk who may only
    // add must not be able to wipe the old rates by re-importing the same date.
    let confirm = body["confirm"].as_bool().unwrap_or(false);
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer.clone(), "payroll", Action::Add).await {
        return resp;
    }
    if confirm {
        if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "payroll", Action::Delete).await {
            return resp;
        }
    }
    let csv_text = match body["csv"].as_str() {
        Some(s) => s,
        None => return NoDataResponse::new("Missing 'csv' field".to_string(), 400).into_response(),
    };
    if csv_text.len() > 1024 * 1024 {
        return NoDataResponse::new("CSV is larger than 1 MiB".to_string(), 400).into_response();
    }
    if csv_text.lines().count() > 5000 {
        return NoDataResponse::new("CSV has more than 5000 rows".to_string(), 400)
            .into_response();
    }

    #[derive(Default)]
    struct EmBlock {
        row: usize,
        employee_id: i32,
        name: String,
        salary_type: String,
        monthly: i64,
        daily: i64,
        hourly: i64,
        effective: String,
        rules: Vec<serde_json::Value>,
        rule_rows: Vec<usize>,
    }
    let mut blocks: Vec<EmBlock> = Vec::new();
    let mut cur: Option<usize> = None;
    // Any EM row seen, even a rejected one: a later RL row then belongs to a
    // block that already has its own error, so it must not add a second one.
    let mut saw_em = false;
    // Rule rows that failed structural checks before a block existed.
    let mut parse_errors: Vec<serde_json::Value> = Vec::new();

    // flexible: the trailing empty RL columns are optional in real files
    // (`...,0` instead of `...,0,,,`), and a rigid reader dropped those rows
    // without a word. A row wider than the header is still an error.
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_text.as_bytes());
    for rec in reader.records() {
        let rec = match rec {
            Ok(r) => r,
            Err(e) => {
                parse_errors.push(serde_json::json!({
                    "row": reader.position().line() as usize,
                    "reason": format!("unreadable CSV row: {e}"),
                }));
                break;
            }
        };
        if rec.is_empty() { continue; }
        let row = rec.position().map(|p| p.line() as usize).unwrap_or(0);
        if rec.len() > 8 {
            parse_errors.push(serde_json::json!({
                "row": row,
                "reason": format!("expected at most 8 columns, found {}", rec.len()),
            }));
            // Do not let following RL rows attach to the previous employee.
            cur = None;
            continue;
        }
        let kind = rec.get(0).unwrap_or("").trim().to_uppercase();
        match kind.as_str() {
            "EM" => {
                saw_em = true;
                // Empty cells default to 0; a non-numeric cell is a hard error,
                // never a silent zero (money must not be guessed).
                let num = |idx: usize, field: &str| -> Result<i64, String> {
                    let raw = rec.get(idx).unwrap_or("").trim();
                    if raw.is_empty() { return Ok(0); }
                    raw.parse::<i64>()
                        .map_err(|_| format!("{field} '{}' is not a number", raw))
                };
                let parsed = (|| -> Result<(i32, i64, i64, i64), String> {
                    let id = match rec.get(1).unwrap_or("").trim() {
                        "" => 0,
                        raw => raw.parse::<i32>()
                            .map_err(|_| format!("employee ID '{}' is not an integer", raw))?,
                    };
                    Ok((id, num(5, "monthly_rate")?, num(6, "daily_rate")?, num(7, "hourly_rate")?))
                })();
                let (id, monthly, daily, hourly) = match parsed {
                    Ok(v) => v,
                    Err(reason) => {
                        parse_errors.push(serde_json::json!({
                            "row": row,
                            "employee_id": rec.get(1).unwrap_or("").trim().parse::<i32>().unwrap_or(0),
                            "reason": reason,
                        }));
                        cur = None;
                        continue;
                    }
                };
                if monthly < 0 || daily < 0 || hourly < 0 {
                    parse_errors.push(serde_json::json!({
                        "row": row,
                        "employee_id": id,
                        "reason": "salary amounts must be >= 0",
                    }));
                    cur = None;
                    continue;
                }
                let stype = rec.get(2).unwrap_or("MONTHLY").trim().to_uppercase();
                if stype != "MONTHLY" && stype != "DAILY" {
                    parse_errors.push(serde_json::json!({
                        "row": row,
                        "employee_id": id,
                        "reason": format!("salary type must be MONTHLY or DAILY, got '{}'", stype),
                    }));
                    cur = None;
                    continue;
                }
                let eff = rec.get(3).unwrap_or("").trim().to_string();
                let name = rec.get(4).unwrap_or("").trim().to_string();
                blocks.push(EmBlock {
                    row,
                    employee_id: id,
                    name,
                    salary_type: if stype == "DAILY" { "daily".to_string() } else { "monthly".to_string() },
                    monthly,
                    daily,
                    hourly,
                    effective: eff,
                    rules: Vec::new(),
                    rule_rows: Vec::new(),
                });
                cur = Some(blocks.len() - 1);
            }
            "RL" => {
                // The export prefixes formula-like text with an apostrophe; drop it
                // again so a downloaded file can be re-imported unchanged.
                let rname = unescape_csv_export(rec.get(1).unwrap_or("").trim()).to_string();
                let rtype = rec.get(2).unwrap_or("ADD").trim().to_uppercase();
                // The template/export carry a second column-header row ("RL,RULE_NAME,..."),
                // which the CSV reader treats as data; skip it explicitly.
                if rname.eq_ignore_ascii_case("RULE_NAME") || rtype == "RULE_TYPE" {
                    continue;
                }
                let Some(i) = cur else {
                    if !saw_em {
                        parse_errors.push(serde_json::json!({
                            "row": row,
                            "reason": "RL row has no preceding EM row",
                        }));
                    }
                    continue;
                };
                let formula = unescape_csv_export(rec.get(3).unwrap_or("").trim()).to_string();
                let is_daily: bool = rec.get(4).unwrap_or("0").trim() == "1";
                if rtype != "ADD" && rtype != "SUB" {
                    parse_errors.push(serde_json::json!({
                        "row": row,
                        "employee_id": blocks[i].employee_id,
                        "rule": rname,
                        "reason": format!("rule type must be ADD or SUB, got '{}'", rtype),
                    }));
                    continue;
                }
                let rule = serde_json::json!({
                    "name": rname,
                    "function": formula,
                    "add": rtype == "ADD",
                    "deduct": rtype == "SUB",
                    "parameter": formula,
                    "daily": is_daily,
                });
                blocks[i].rules.push(rule);
                blocks[i].rule_rows.push(row);
            }
            _ => {}
        }
    }

    // Reject malformed rules before touching the DB: the same validator the
    // single-salary assign path uses, applied per rule so the CSV row is known.
    let mut errors = parse_errors;
    for b in &blocks {
        for (i, rule) in b.rules.iter().enumerate() {
            let row = b.rule_rows.get(i).copied().unwrap_or(b.row);
            if let Err(e) = validate_salary_rules(&serde_json::Value::Array(vec![rule.clone()])) {
                errors.push(serde_json::json!({
                    "row": row,
                    "employee_id": b.employee_id,
                    "rule": rule["name"].as_str().unwrap_or(""),
                    "reason": e.replacen("rules_json[0]: ", "", 1),
                }));
            }
        }
    }
    if !errors.is_empty() {
        return actix_web::HttpResponse::BadRequest().json(serde_json::json!({
            "code": 400,
            "message": "Import validation failed",
            "error": "payroll.importInvalid",
            "data": { "errors": errors },
        }));
    }

    // Validate employees and dates the same way rules are validated: one bad row
    // rejects the whole file. Half-importing a file left employees without a
    // salary while the response said "Imported 99 employees".
    let mut valid_blocks: Vec<EmBlock> = Vec::new();
    for b in blocks {
        if b.employee_id <= 0 {
            errors.push(serde_json::json!({
                "row": b.row,
                "reason": format!("employee ID '{}' is missing", b.name),
            }));
            continue;
        }
        let exists: Option<i32> = sqlx::query_scalar("SELECT employee_id FROM employee WHERE employee_id = $1")
            .bind(b.employee_id)
            .fetch_optional(&pool.db)
            .await
            .unwrap_or(None);
        if exists.is_none() {
            errors.push(serde_json::json!({
                "row": b.row,
                "employee_id": b.employee_id,
                "reason": format!("employee {} does not exist", b.employee_id),
            }));
            continue;
        }
        let eff = match parse_effective_date(&b.effective) {
            Some(d) => d,
            None => {
                errors.push(serde_json::json!({
                    "row": b.row,
                    "employee_id": b.employee_id,
                    "reason": format!("effective date '{}' is not a valid date", b.effective),
                }));
                continue;
            }
        };
        let mut b = b;
        b.effective = eff.to_string();
        valid_blocks.push(b);
    }
    if !errors.is_empty() {
        return actix_web::HttpResponse::BadRequest().json(serde_json::json!({
            "code": 400,
            "message": "Import validation failed",
            "error": "payroll.importInvalid",
            "data": { "errors": errors },
        }));
    }

    // Two blocks for the same employee+date would overwrite each other inside this
    // file, and the DB duplicate scan would report only one of them.
    let mut seen_pairs: std::collections::HashSet<(i32, String)> = std::collections::HashSet::new();
    let mut in_file_dupes: Vec<serde_json::Value> = Vec::new();
    for b in &valid_blocks {
        if !seen_pairs.insert((b.employee_id, b.effective.clone())) {
            in_file_dupes.push(serde_json::json!({
                "row": b.row,
                "employee_id": b.employee_id,
                "reason": format!("employee {} and date {} appear more than once", b.employee_id, b.effective),
            }));
        }
    }
    if !in_file_dupes.is_empty() {
        return actix_web::HttpResponse::BadRequest().json(serde_json::json!({
            "code": 400,
            "message": "Import validation failed",
            "error": "payroll.importInvalid",
            "data": { "errors": in_file_dupes },
        }));
    }

    // Detect duplicates.
    let mut duplicates: Vec<serde_json::Value> = Vec::new();
    for b in &valid_blocks {
        let eff: chrono::NaiveDate = b.effective.parse().unwrap();
        let exists: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM salaries WHERE employee_id = $1 AND effective_date = $2 LIMIT 1",
        )
        .bind(b.employee_id)
        .bind(eff)
        .fetch_optional(&pool.db)
        .await
        .unwrap_or(None);
        if exists.is_some() {
            log::info!("DEBUG import: duplicate found for employee_id={}", b.employee_id);
            duplicates.push(serde_json::json!({"id": b.employee_id, "name": b.name, "effective_date": b.effective}));
        }
    }

    if !confirm && !duplicates.is_empty() {
        return actix_web::HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "Confirm replace",
            "data": {
                "needs_confirm": true,
                "duplicates": duplicates,
                "missing": [],
            }
        }));
    }

    // One transaction: DELETE then INSERT per block. Without it a failed INSERT
    // leaves the employee with no salary row while the response says success.
    // In-file duplicates are rejected above, so a block never deletes a row that
    // a later block in the same file would need.
    let mut tx = match pool.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("import: begin failed: {}", e);
            return NoDataResponse::new("Database error".to_string(), 500).into_response();
        }
    };
    let mut imported_emp = 0i32;
    let mut imported_rules = 0i32;
    for b in &valid_blocks {
        let eff: chrono::NaiveDate = b.effective.parse().unwrap();
        if let Err(e) = sqlx::query("DELETE FROM salaries WHERE employee_id = $1 AND effective_date = $2")
            .bind(b.employee_id)
            .bind(eff)
            .execute(&mut *tx)
            .await
        {
            log::error!("import: delete for employee {} failed: {}", b.employee_id, e);
            return NoDataResponse::new("Import failed, nothing was saved".to_string(), 500)
                .into_response();
        }
        let rules_value = serde_json::to_value(&b.rules).unwrap_or(serde_json::json!([]));
        let insert = sqlx::query(
            r#"INSERT INTO salaries
               (employee_id, salary_type, monthly_rate, daily_salary, hourly_rate, basic_salary, department, rules_json, ot_eligible, effective_date)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
        )
        .bind(b.employee_id)
        .bind(&b.salary_type)
        .bind(b.monthly)
        .bind(b.daily)
        .bind(b.hourly)
        .bind(0i64)
        .bind(Option::<String>::None)
        .bind(&rules_value)
        .bind(true)
        .bind(eff)
        .execute(&mut *tx)
        .await;
        if let Err(e) = insert {
            log::error!("import: insert for employee {} failed: {}", b.employee_id, e);
            return NoDataResponse::new("Import failed, nothing was saved".to_string(), 500)
                .into_response();
        }
        imported_emp += 1;
        imported_rules += b.rules.len() as i32;
    }
    if let Err(e) = tx.commit().await {
        log::error!("import: commit failed: {}", e);
        return NoDataResponse::new("Import failed, nothing was saved".to_string(), 500)
            .into_response();
    }

    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "message": format!("Imported {} employees, {} rules", imported_emp, imported_rules),
        "data": {
            "needs_confirm": false,
            "imported_employees": imported_emp,
            "imported_rules": imported_rules,
            "missing": [],
            "duplicates_replaced": duplicates.len(),
        }
    }))
}
