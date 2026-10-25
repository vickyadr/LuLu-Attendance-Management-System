use std::collections::HashMap;

use crate::{
    models::m_employee::*,
    receiver::{r_csv_import::*, r_employee::*},
    utility::{
        authorization::{check_action_access, Action},
        stor::{AppState, GenericResponse, IntoResponse, KeyValResponse, NoDataResponse},
    },
};

use actix_web::{
    Responder, get, post,
    web::{self, ReqData},
};

#[derive(Debug, serde::Deserialize)]
pub struct EmployeeListQuery {
    pub q: Option<String>,
    pub department: Option<String>,
    pub status: Option<i32>,
    /// Absent means "every employee": the dashboard, payroll pickers and the
    /// forms still need the whole table, only the employee page pages.
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

const EMPLOYEE_PAGE_SELECT: &str = r#"SELECT * FROM employee
    WHERE ($1::TEXT IS NULL OR trim(employee_fname || ' ' || COALESCE(employee_lname, '')) ILIKE '%' || $1 || '%'
        OR COALESCE(employee_department, '') ILIKE '%' || $1 || '%'
        OR COALESCE(employee_address, '') ILIKE '%' || $1 || '%')
      AND ($2::TEXT IS NULL OR employee_department = $2)
      AND ($3::INTEGER IS NULL OR employee_status = $3)
    ORDER BY employee_fname ASC, employee_id ASC LIMIT $4 OFFSET $5"#;

const EMPLOYEE_PAGE_COUNT: &str = r#"SELECT count(*) FROM employee
    WHERE ($1::TEXT IS NULL OR trim(employee_fname || ' ' || COALESCE(employee_lname, '')) ILIKE '%' || $1 || '%'
        OR COALESCE(employee_department, '') ILIKE '%' || $1 || '%'
        OR COALESCE(employee_address, '') ILIKE '%' || $1 || '%')
      AND ($2::TEXT IS NULL OR employee_department = $2)
      AND ($3::INTEGER IS NULL OR employee_status = $3)"#;

#[get("/employee/list")]
pub async fn employee_list(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    query: web::Query<EmployeeListQuery>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "employee", Action::View).await {
        return resp;
    }

    if let Some(limit) = query.limit {
        let limit = limit.clamp(1, 100);
        let offset = query.offset.unwrap_or(0).max(0);
        let q = query
            .q
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let department = query
            .department
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        let rows = sqlx::query_as::<_, Employee>(EMPLOYEE_PAGE_SELECT)
            .bind(&q)
            .bind(&department)
            .bind(query.status)
            .bind(limit)
            .bind(offset)
            .fetch_all(&pool.db)
            .await;

        let total = sqlx::query_scalar::<_, i64>(EMPLOYEE_PAGE_COUNT)
            .bind(&q)
            .bind(&department)
            .bind(query.status)
            .fetch_one(&pool.db)
            .await;

        let stats = sqlx::query(
            r#"SELECT count(*) AS total,
                      count(*) FILTER (WHERE employee_status = 0) AS active,
                      count(*) FILTER (WHERE employee_status <> 0) AS inactive
               FROM employee"#,
        )
        .fetch_one(&pool.db)
        .await;

        let departments = sqlx::query_scalar::<_, String>(
            r#"SELECT DISTINCT employee_department FROM employee
               WHERE employee_department IS NOT NULL AND employee_department <> ''
               ORDER BY employee_department"#,
        )
        .fetch_all(&pool.db)
        .await;

        return match (rows, total, stats, departments) {
            (Ok(rows), Ok(total), Ok(stats), Ok(departments)) => {
                use sqlx::Row;
                GenericResponse::<serde_json::Value>::ok(
                    vec![serde_json::json!({
                        "items": rows,
                        "total": total,
                        "all_total": stats.get::<i64, _>("total"),
                        "active": stats.get::<i64, _>("active"),
                        "inactive": stats.get::<i64, _>("inactive"),
                        "departments": departments,
                    })],
                    "ok".to_string(),
                )
                .into_response()
            }
            (Err(e), _, _, _) | (_, Err(e), _, _) | (_, _, Err(e), _) | (_, _, _, Err(e)) => {
                log::error!("Error: {}", e);
                NoDataResponse::new("Internal server error".to_string(), 500).into_response()
            }
        };
    }

    match sqlx::query_as::<_, Employee>(
        r#"SELECT * FROM employee ORDER BY employee_fname ASC"#,
    )
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => GenericResponse::<Employee>::ok(data, "ok".to_string()).into_response(),
        Err(e) => {
            log::error!("Error: {}", e);

            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct UnregisteredQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[get("/employee/unregistered")]
pub async fn employee_unregistered(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    query: web::Query<UnregisteredQuery>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "employee", Action::View).await {
        return resp;
    }

    // A punch from a device-side user id that has no employee row. These cannot
    // appear in a report: reports filter by date and by an employee with a
    // schedule, and an unregistered id has neither. This list is the only place
    // they are visible, and it is where the operator registers them.
    //
    // `device_timezone` is the timezone the owning device stamps its punches in,
    // so the UI renders first/last punch on the device's clock instead of the
    // browser's. An uid punched by more than one device takes the newest device.
    let (sql, limit, offset) = match query.limit {
        Some(limit) => (
            r#"SELECT e.enroll_employee_id AS device_uid,
                      count(*) AS punch_count,
                      min(e.enroll_time) AS first_punch,
                      max(e.enroll_time) AS last_punch,
                      (array_agg(d.device_timezone ORDER BY e.enroll_time DESC))[1] AS timezone
               FROM enrolls e
               LEFT JOIN employee em ON em.employee_device_uid = e.enroll_employee_id
               LEFT JOIN devices d ON d.device_sn = e.enroll_device_sn
               WHERE em.employee_device_uid IS NULL
               GROUP BY e.enroll_employee_id
               ORDER BY max(e.enroll_time) DESC, e.enroll_employee_id ASC
               LIMIT $1 OFFSET $2"#,
            limit.clamp(1, 200),
            query.offset.unwrap_or(0).max(0),
        ),
        None => (
            r#"SELECT e.enroll_employee_id AS device_uid,
                      count(*) AS punch_count,
                      min(e.enroll_time) AS first_punch,
                      max(e.enroll_time) AS last_punch,
                      (array_agg(d.device_timezone ORDER BY e.enroll_time DESC))[1] AS timezone
               FROM enrolls e
               LEFT JOIN employee em ON em.employee_device_uid = e.enroll_employee_id
               LEFT JOIN devices d ON d.device_sn = e.enroll_device_sn
               WHERE em.employee_device_uid IS NULL
               GROUP BY e.enroll_employee_id
               ORDER BY max(e.enroll_time) DESC"#,
            0,
            0,
        ),
    };

    let rows = if query.limit.is_some() {
        sqlx::query_as::<_, UnregisteredUser>(sql)
            .bind(limit)
            .bind(offset)
            .fetch_all(&pool.db)
            .await
    } else {
        sqlx::query_as::<_, UnregisteredUser>(sql)
            .fetch_all(&pool.db)
            .await
    };

    match rows
    {
        Ok(data) => {
            if query.limit.is_some() {
                let total = sqlx::query_scalar::<_, i64>(
                    r#"SELECT count(DISTINCT e.enroll_employee_id)
                       FROM enrolls e
                       LEFT JOIN employee em ON em.employee_device_uid = e.enroll_employee_id
                       WHERE em.employee_device_uid IS NULL"#,
                )
                .fetch_one(&pool.db)
                .await;
                match total {
                    Ok(total) => GenericResponse::<serde_json::Value>::ok(
                        vec![serde_json::json!({ "items": data, "total": total })],
                        "ok".to_string(),
                    )
                    .into_response(),
                    Err(e) => {
                        log::error!("Error counting unregistered device users: {}", e);
                        NoDataResponse::new("Internal server error".to_string(), 500).into_response()
                    }
                }
            } else {
                GenericResponse::<UnregisteredUser>::ok(data, "ok".to_string()).into_response()
            }
        }
        Err(e) => {
            log::error!("Error listing unregistered device users: {}", e);

            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

#[post("/employee/add")]
pub async fn employee_add(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverEmployee>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "employee", Action::Add).await {
        return resp;
    }

    match body.fname.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("first_name", "Too short".to_string());
            }
        }
        None => {
            data.insert("first_name", "Field required".to_string());
        }
    }

    match body.department.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("department", "Too short".to_string());
            }
        }
        None => {
            data.insert("department", "Field required".to_string());
        }
    }

    match body.address.clone() {
        Some(o) => {
            if o.len().ge(&160) {
                data.insert("address", "Max 160 character".to_string());
            }
        }
        None => {
            data.insert("location", "Field required".to_string());
        }
    }

    // Every employee must own a machine number. An unlinked employee means their
    // punches resolve to nobody and land in the unregistered list, which is the
    // exact failure this field exists to prevent.
    match body.device_uid {
        Some(v) if v >= 1 => {}
        _ => {
            data.insert("device_uid", "Field required".to_string());
        }
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(
            data,
            "There's was error while adding employee".to_string(),
            403,
        )
        .into_response();
    }

    match sqlx::query_as::<_, Employee>(r#"INSERT INTO employee (employee_fname, employee_lname, employee_department, employee_address, employee_status, employee_schedule_id, employee_device_uid) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"#)
                .bind(body.fname.clone())
                .bind(body.lname.clone().unwrap_or_default())
                .bind(body.department.clone())
                .bind(body.address.clone())
                .bind(body.status)
                .bind(body.schedule_id)
                .bind(body.device_uid)
                .fetch_one(&pool.db)
                .await
        {
            Ok(data) => GenericResponse::<Employee>::single(
                                    data,
                                    "New employee registered !!!".to_string()
                                ).into_response(),
            Err(e) => {
                log::error!("{}", e);
                // Only a constraint violation carries a DB message. A pool timeout
                // or a broken connection is not a database error, and unwrapping
                // here panicked the worker whenever Postgres was briefly busy.
                let code = e.as_database_error().map(|d| d.message()).unwrap_or("");
                if code.contains("duplicate key value violates") {
                    return NoDataResponse::new(
                        "This employee have registered!".to_string(),
                        405
                    ).into_response();
                }
                NoDataResponse::new(
                    "Internal error, please try again!".to_string(),
                    500
                ).into_response()
            }
        }
}

#[get("/employee/delete/{id}")]
pub async fn employee_delete(
    path: web::Path<i32>,
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let id = path.into_inner();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "employee", Action::Delete).await
    {
        return resp;
    }

    match sqlx::query(r#"DELETE FROM employee WHERE employee_id=$1"#)
        .bind(id)
        .execute(&pool.db)
        .await
    {
        Ok(_) => NoDataResponse::ok("Employee deleted".to_string()).into_response(),
        Err(_) => NoDataResponse::new("Employee not found".to_string(), 404).into_response(),
    }
}

#[post("/employee/edit")]
pub async fn employee_edit(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverEmployee>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "employee", Action::Edit).await {
        return resp;
    }

    match body.fname.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("first_name", "Too short".to_string());
            }
        }
        None => {
            data.insert("first_name", "Field required".to_string());
        }
    }

    match body.department.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("department", "Too short".to_string());
            }
        }
        None => {
            data.insert("department", "Field required".to_string());
        }
    }

    match body.address.clone() {
        Some(o) => {
            if o.len().ge(&160) {
                data.insert("address", "Max 160 character".to_string());
            }
        }
        None => {
            data.insert("location", "Field required".to_string());
        }
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(
            data,
            "There's was error while adding controller".to_string(),
            403,
        )
        .into_response();
    }

    // Re-linking an employee that already has punch history would orphan those
    // punches retroactively: the old uid's rows stop resolving to this employee
    // in every report that goes through the join, so historical attendance
    // disappears the moment the link is changed. Refuse instead of silently
    // rewriting history.
    //
    // ponytail: clearing the link to NULL is allowed even with history (the
    // punches become visible in the unregistered list again, which is honest).
    // Only a *different* uid is blocked. If a real device renumber needs this,
    // add a deliberate reassignment endpoint that migrates enrolls rows too.
    let stored_uid: Option<i32> = match sqlx::query_scalar::<_, Option<i32>>(
        "SELECT employee_device_uid FROM employee WHERE employee_id = $1",
    )
    .bind(body.id)
    .fetch_one(&pool.db)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to read stored device_uid: {}", e);
            return NoDataResponse::new("Internal server error".to_string(), 500).into_response();
        }
    };
    let has_punches: bool = match sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM enrolls WHERE enroll_employee_id = $1 LIMIT 1",
    )
    .bind(stored_uid)
    .fetch_one(&pool.db)
    .await
    {
        Ok(c) => c > 0,
        Err(e) => {
            log::error!("Failed to count punches for device_uid: {}", e);
            return NoDataResponse::new("Internal server error".to_string(), 500).into_response();
        }
    };
    if has_punches && body.device_uid != stored_uid {
        return NoDataResponse::coded(
            "employee.deviceUidHasHistory",
            "This User ID has attendance history and cannot be changed. Create a new \
             employee to claim the machine's number instead."
                .to_string(),
            409,
        )
        .into_response();
    }

    // Preserve device_uid if omitted in request (None means don't change).
    let device_uid_to_update = body.device_uid.or(stored_uid);
    
    match sqlx::query_as::<_, Employee>(r#"UPDATE employee SET employee_fname = $1 , employee_lname =$2 , employee_department = $3, employee_address = $4, employee_status = $5, employee_schedule_id = $6, employee_device_uid = $7 WHERE employee_id = $8 RETURNING *"#)
                    .bind(body.fname.clone())
                    .bind(body.lname.clone().unwrap_or_default())
                    .bind(body.department.clone())
                    .bind(body.address.clone())
                    .bind(body.status)
                    .bind(body.schedule_id)
                    .bind(device_uid_to_update)
                    .bind(body.id)
                    .fetch_one(&pool.db)
                    .await
            {
            Ok(data) => GenericResponse::<Employee>::single(
                                    data,
                                    "Employee data updated !!!".to_string()
                                ).into_response(),
            Err(e) => {
                log::error!("{}", e);
                NoDataResponse::new(
                    "Internal error, please try again!".to_string(),
                    500
                ).into_response()
            }
        }
}

#[post("/employee/import")]
pub async fn employee_import(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    body: web::Json<serde_json::Value>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "employee", Action::Add).await {
        return resp;
    }

    let csv_text = match body["csv"].as_str() {
        Some(s) => s,
        None => return NoDataResponse::new("Missing 'csv' field".to_string(), 400).into_response(),
    };

    let mut result = ImportResult::default();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_text.as_bytes());

    // Start a transaction: if any insert fails, rollback everything so the
    // operator sees a clean failure, not a half-imported batch.
    let mut tx = match pool.db.begin().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to begin transaction: {}", e);
            return NoDataResponse::new("Transaction error".to_string(), 500).into_response();
        }
    };

    // ponytail: hardcoded header names; add headermap detection if operators need
    // flexible column order. For now, they follow the template.
    for (row_idx, row_result) in reader.deserialize().enumerate() {
        let row_num = row_idx + 2; // +1 for header, +1 for human-readable 1-indexing
        let row: EmployeeCsvRow = match row_result {
            Ok(r) => r,
            Err(e) => {
                result.errors.push(ImportError {
                    row: row_num,
                    field: "*".to_string(),
                    reason: format!("Parse error: {}", e),
                });
                continue;
            }
        };

        // Parse once and bail on a bad id: everything below depends on it, and the
        // other columns are reported individually anyway.
        let device_uid: i32 = match row.device_uid.trim().parse() {
            Ok(n) if n >= 1 => n,
            _ => {
                result.errors.push(ImportError {
                    row: row_num,
                    field: "id".to_string(),
                    reason: "Must be a number >= 1".to_string(),
                });
                continue;
            }
        };

        let mut errs = Vec::new();

        if row.first_name.trim().len() < 3 {
            errs.push(("first_name", "Min 3 characters"));
        }
        if row.last_name.trim().len() < 3 {
            errs.push(("last_name", "Min 3 characters"));
        }
        if row.department.trim().len() < 3 {
            errs.push(("department", "Min 3 characters"));
        }
        if row.address.trim().len() > 160 {
            errs.push(("address", "Max 160 characters"));
        }

        // Look up schedule by name (case-insensitive)
        let schedule_id: i32 = match sqlx::query_scalar::<_, i32>(
            "SELECT schedule_id FROM schedules WHERE lower(schedule_name) = lower($1) LIMIT 1",
        )
        .bind(row.schedule.trim())
        .fetch_optional(&mut *tx)
        .await
        {
            Ok(Some(id)) => id,
            Ok(None) => {
                errs.push(("schedule", "Schedule not found"));
                0 // never inserted: errs is non-empty, so the loop continues first
            }
            Err(e) => {
                log::error!("Schedule lookup error: {}", e);
                return NoDataResponse::new("Internal server error".to_string(), 500).into_response();
            }
        };

        if !errs.is_empty() {
            for (field, reason) in errs {
                result.errors.push(ImportError {
                    row: row_num,
                    field: field.to_string(),
                    reason: reason.to_string(),
                });
            }
            continue;
        }

        // Check unique constraint on device_uid (allow only NEW uids, or existing with no punches)
        let existing: Option<(i32, i32)> = match sqlx::query_as::<_, (i32, i32)>(
            "SELECT e.employee_id, COALESCE((SELECT 1 FROM enrolls WHERE enroll_employee_id=e.employee_device_uid LIMIT 1), 0) FROM employee e WHERE e.employee_device_uid = $1 LIMIT 1",
        )
        .bind(device_uid)
        .fetch_optional(&mut *tx)
        .await
        {
            Ok(x) => x,
            Err(e) => {
                log::error!("Unique check error: {}", e);
                result.errors.push(ImportError {
                    row: row_num,
                    field: "id".to_string(),
                    reason: "Lookup error".to_string(),
                });
                continue;
            }
        };

        if let Some((_, has_punch)) = existing {
            if has_punch > 0 {
                result.errors.push(ImportError {
                    row: row_num,
                    field: "id".to_string(),
                    reason: "ID already linked to employee with punch history".to_string(),
                });
                continue;
            }
        }

        // Insert
        match sqlx::query::<_>(
            "INSERT INTO employee (employee_fname, employee_lname, employee_department, employee_address, employee_status, employee_schedule_id, employee_device_uid) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (employee_device_uid) WHERE employee_device_uid IS NOT NULL DO NOTHING",
        )
        .bind(row.first_name.trim())
        .bind(row.last_name.trim())
        .bind(row.department.trim())
        .bind(row.address.trim())
        .bind(0) // status active
        .bind(schedule_id)
        .bind(device_uid)
        .execute(&mut *tx)
        .await
        {
            Ok(_) => result.inserted += 1,
            Err(e) => {
                // A rejected row here is a database failure, not bad input: the
                // per-row validation above already reported the bad rows and left
                // the transaction untouched. Continuing would commit a partial
                // batch, so drop the whole import and let the operator fix the
                // cause rather than discover half a roster later.
                log::error!("Import insert failed at row {}: {}", row_num, e);
                return NoDataResponse::new(
                    format!("Import failed at row {row_num}: {e}"),
                    500,
                )
                .into_response();
            }
        }
    }

    if let Err(e) = tx.commit().await {
        log::error!("Import commit failed: {}", e);
        return NoDataResponse::new("Import commit failed".to_string(), 500).into_response();
    }

    let inserted = result.inserted;
    GenericResponse::<ImportResult>::single(result, format!("Imported {} employees", inserted)).into_response()
}
