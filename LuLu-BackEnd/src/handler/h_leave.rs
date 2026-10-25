use crate::{
    models::m_leave::*,
    utility::{
        authorization::{Action, check_action_access, get_session_user},
        stor::{AppState, GenericResponse, IntoResponse, KeyValResponse, NoDataResponse},
    },
};
use actix_web::{
    Responder, get, post,
    web::{self, ReqData},
};
use chrono::NaiveDate;
use sqlx::Row;
use std::collections::HashMap;

/// The kinds a request may carry, as machine values. Mirrors the
/// `leave_requests_kind_check` constraint in the migration; the UI maps each to a
/// label, so the two lists must stay in step.
const KINDS: [&str; 3] = ["cuti", "izin", "sakit"];

fn parse_date(raw: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").ok()
}

#[derive(Debug, serde::Deserialize)]
pub struct LeaveListQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub employee_id: Option<i32>,
    pub q: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[get("/leave/list")]
pub async fn leave_list(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    query: web::Query<LeaveListQuery>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer.clone(), "leave", Action::View).await {
        return resp;
    }

    // Both bounds and the employee filter are optional, so the panel can show
    // every request in a month, or one person's history across all time.
    let from = query.from.as_deref().and_then(parse_date);
    let to = query.to.as_deref().and_then(parse_date);
    let employee = query.employee_id.filter(|id| *id > 0);
    let status = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let name = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    const LEAVE_SELECT: &str = r#"SELECT l.leave_id, l.leave_employee_id, l.leave_date, l.leave_kind,
                  l.leave_reason, l.leave_status, l.leave_source,
                  l.request_by, l.decided_by,
                  COALESCE(e.employee_fname, '') AS employee_fname,
                  COALESCE(e.employee_lname, '') AS employee_lname
           FROM leave_requests l
           LEFT JOIN employee e ON e.employee_id = l.leave_employee_id
           WHERE ($1::DATE IS NULL OR l.leave_date >= $1)
             AND ($2::DATE IS NULL OR l.leave_date <= $2)
             AND ($3::INTEGER IS NULL OR l.leave_employee_id = $3)
             AND ($4::TEXT IS NULL OR l.leave_status = $4)
             AND ($5::TEXT IS NULL OR trim(COALESCE(e.employee_fname, '') || ' ' || COALESCE(e.employee_lname, '')) ILIKE '%' || $5 || '%')
           ORDER BY l.leave_date DESC, l.leave_id DESC"#;
    const LEAVE_SELECT_PAGED: &str = r#"SELECT l.leave_id, l.leave_employee_id, l.leave_date, l.leave_kind,
                  l.leave_reason, l.leave_status, l.leave_source,
                  l.request_by, l.decided_by,
                  COALESCE(e.employee_fname, '') AS employee_fname,
                  COALESCE(e.employee_lname, '') AS employee_lname
           FROM leave_requests l
           LEFT JOIN employee e ON e.employee_id = l.leave_employee_id
           WHERE ($1::DATE IS NULL OR l.leave_date >= $1)
             AND ($2::DATE IS NULL OR l.leave_date <= $2)
             AND ($3::INTEGER IS NULL OR l.leave_employee_id = $3)
             AND ($4::TEXT IS NULL OR l.leave_status = $4)
             AND ($5::TEXT IS NULL OR trim(COALESCE(e.employee_fname, '') || ' ' || COALESCE(e.employee_lname, '')) ILIKE '%' || $5 || '%')
           ORDER BY l.leave_date DESC, l.leave_id DESC
           LIMIT $6 OFFSET $7"#;
    const LEAVE_STATUS_COUNTS: &str = r#"SELECT
             count(*) FILTER (WHERE l.leave_status = 'pending') AS pending,
             count(*) FILTER (WHERE l.leave_status = 'approved') AS approved,
             count(*) FILTER (WHERE l.leave_status = 'rejected') AS rejected
           FROM leave_requests l
           LEFT JOIN employee e ON e.employee_id = l.leave_employee_id
           WHERE ($1::DATE IS NULL OR l.leave_date >= $1)
             AND ($2::DATE IS NULL OR l.leave_date <= $2)
             AND ($3::INTEGER IS NULL OR l.leave_employee_id = $3)
             AND ($4::TEXT IS NULL OR trim(COALESCE(e.employee_fname, '') || ' ' || COALESCE(e.employee_lname, '')) ILIKE '%' || $4 || '%')"#;
    const LEAVE_COUNT: &str = r#"SELECT count(*)
           FROM leave_requests l
           LEFT JOIN employee e ON e.employee_id = l.leave_employee_id
           WHERE ($1::DATE IS NULL OR l.leave_date >= $1)
             AND ($2::DATE IS NULL OR l.leave_date <= $2)
             AND ($3::INTEGER IS NULL OR l.leave_employee_id = $3)
             AND ($4::TEXT IS NULL OR l.leave_status = $4)
             AND ($5::TEXT IS NULL OR trim(COALESCE(e.employee_fname, '') || ' ' || COALESCE(e.employee_lname, '')) ILIKE '%' || $5 || '%')"#;

    let (rows, total) = match query.limit {
        Some(limit) => {
            let limit = limit.clamp(1, 200);
            let offset = query.offset.unwrap_or(0).max(0);
            let rows = sqlx::query(LEAVE_SELECT_PAGED)
                .bind(from)
                .bind(to)
                .bind(employee)
                .bind(&status)
                .bind(&name)
                .bind(limit)
                .bind(offset)
                .fetch_all(&pool.db)
                .await;
            let total = sqlx::query_scalar::<_, i64>(LEAVE_COUNT)
                .bind(from)
                .bind(to)
                .bind(employee)
                .bind(&status)
                .bind(&name)
                .fetch_one(&pool.db)
                .await;
            let counts = sqlx::query(LEAVE_STATUS_COUNTS)
                .bind(from)
                .bind(to)
                .bind(employee)
                .bind(&name)
                .fetch_one(&pool.db)
                .await;
            (rows, Some((total, counts)))
        }
        None => (
            sqlx::query(LEAVE_SELECT)
                .bind(from)
                .bind(to)
                .bind(employee)
                .bind(&status)
                .bind(&name)
                .fetch_all(&pool.db)
                .await,
            None,
        ),
    };

    match rows {
        Ok(rs) => {
            // The employee name rides along so the panel never has to join two
            // responses client-side, which is where a stale filter would show a
            // request under the wrong person.
            let data: Vec<LeaveListRow> = rs
                .into_iter()
                .map(|r| LeaveListRow {
                    leave_id: r.get("leave_id"),
                    leave_employee_id: r.get("leave_employee_id"),
                    leave_date: r.get("leave_date"),
                    leave_kind: r.get("leave_kind"),
                    leave_reason: r.get("leave_reason"),
                    leave_status: r.get("leave_status"),
                    leave_source: r.get("leave_source"),
                    request_by: r.get("request_by"),
                    decided_by: r.get("decided_by"),
                    employee_fname: r.get("employee_fname"),
                    employee_lname: r.get("employee_lname"),
                })
                .collect();
            if let Some((total, counts)) = total {
                match (total, counts) {
                    (Ok(total), Ok(counts)) => {
                        use sqlx::Row;
                        GenericResponse::<serde_json::Value>::ok(
                            vec![serde_json::json!({
                                "items": data,
                                "total": total,
                                "pending": counts.get::<i64, _>("pending"),
                                "approved": counts.get::<i64, _>("approved"),
                                "rejected": counts.get::<i64, _>("rejected"),
                            })],
                            "OK".to_string(),
                        )
                        .into_response()
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        log::error!("{}", e);
                        NoDataResponse::new("Internal server error".to_string(), 500).into_response()
                    }
                }
            } else {
                GenericResponse::<LeaveListRow>::ok(data, "OK".to_string()).into_response()
            }
        }
        // An empty list is the normal state of a fresh install; a missing table
        // is a failed migration. Neither is a client error, so the panel shows
        // "nothing requested yet" rather than an error.
        Err(e) => {
            log::error!("{}", e);
            GenericResponse::<LeaveListRow>::ok(Vec::new(), "OK".to_string()).into_response()
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct LeaveListRow {
    leave_id: i32,
    leave_employee_id: i32,
    leave_date: chrono::NaiveDate,
    leave_kind: String,
    leave_reason: String,
    leave_status: String,
    leave_source: String,
    request_by: String,
    decided_by: String,
    employee_fname: String,
    employee_lname: String,
}

#[get("/leave/employee-options")]
pub async fn leave_employee_options(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "leave", Action::View).await {
        return resp;
    }

    // The name picker for the request form. Approving leave names a person, so
    // this page legitimately needs the employee list, but it must NOT be the full
    // `employee` page: `employee/list` returns addresses and other personal
    // columns, which an attendance administrator has no reason to read in order
    // to pick a day off. Selecting only id, name and department keeps the
    // permission honest, so `leave` can be granted without `employee`.
    let rows = sqlx::query(
        r#"SELECT employee_id AS id,
                  COALESCE(employee_fname, '') AS first_name,
                  COALESCE(employee_lname, '') AS last_name,
                  COALESCE(employee_department, '') AS department
           FROM employee
           ORDER BY employee_fname ASC"#,
    )
    .fetch_all(&pool.db)
    .await;

    match rows {
        Ok(rs) => {
            let data: Vec<EmployeeOption> = rs
                .into_iter()
                .map(|r| EmployeeOption {
                    id: r.get("id"),
                    first_name: r.get("first_name"),
                    last_name: r.get("last_name"),
                    department: r.get("department"),
                })
                .collect();
            GenericResponse::<EmployeeOption>::ok(data, "OK".to_string()).into_response()
        }
        Err(e) => {
            log::error!("{}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct EmployeeOption {
    id: i32,
    first_name: String,
    last_name: String,
    department: String,
}

#[post("/leave/edit")]
pub async fn leave_edit(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverLeave>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer.clone(), "leave", Action::Add).await {
        return resp;
    }

    let mut errs: HashMap<String, String> = HashMap::new();

    let Some(employee_id) = body.leave_employee_id.filter(|id| *id > 0) else {
        errs.insert("leave_employee_id".to_string(), "Pick an employee".to_string());
        return KeyValResponse::<String, String>::new(
            errs,
            "There's was error while saving leave request".to_string(),
            403,
        )
        .into_response();
    };

    let Some(date_raw) = body.leave_date.as_deref() else {
        return NoDataResponse::new("A date is required".to_string(), 400).into_response();
    };

    let Some(date) = parse_date(date_raw) else {
        errs.insert("leave_date".to_string(), "Use the YYYY-MM-DD format".to_string());
        return KeyValResponse::<String, String>::new(
            errs,
            "There's was error while saving leave request".to_string(),
            403,
        )
        .into_response();
    };

    let kind = body
        .leave_kind
        .as_deref()
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .unwrap_or("izin");

    if !KINDS.contains(&kind) {
        errs.insert(
            "leave_kind".to_string(),
            "Use cuti, izin or sakit".to_string(),
        );
    }

    if !errs.is_empty() {
        return KeyValResponse::<String, String>::new(
            errs,
            "There's was error while saving leave request".to_string(),
            403,
        )
        .into_response();
    }

    // Reject a request for an employee that does not exist here rather than
    // letting the foreign key surface as an opaque 500.
    let exists: Option<i32> = sqlx::query_scalar("SELECT employee_id FROM employee WHERE employee_id = $1")
        .bind(employee_id)
        .fetch_optional(&pool.db)
        .await
        .ok()
        .flatten();

    if exists.is_none() {
        let mut errs: HashMap<String, String> = HashMap::new();
        errs.insert("leave_employee_id".to_string(), "Employee not found".to_string());
        return KeyValResponse::<String, String>::new(
            errs,
            "There's was error while saving leave request".to_string(),
            403,
        )
        .into_response();
    }

    // The requester is recorded for the audit trail, so a panel can show who
    // filed it rather than an unattributable row.
    let requester = get_session_user(pool.db.clone(), bearer).await.unwrap_or_default();

    // Insert-or-update on (employee, date): the unique constraint makes a second
    // request for the same day a conflict, and the operator's intent when they
    // re-submit is to correct the previous one, not to file a duplicate.
    let inserted = sqlx::query(
        r#"INSERT INTO leave_requests
             (leave_employee_id, leave_date, leave_kind, leave_reason,
              leave_status, leave_source, request_by)
           VALUES ($1, $2, $3, $4, 'pending', 'manual', $5)
           ON CONFLICT (leave_employee_id, leave_date) DO UPDATE
             SET leave_kind = EXCLUDED.leave_kind,
                 leave_reason = EXCLUDED.leave_reason,
                 leave_status = 'pending',
                 decided_by = '',
                 update_at = EXTRACT(EPOCH FROM NOW())::BIGINT
             -- Only a pending request can be edited back to pending. Resetting a
             -- decided one would clear the decision and let it be withdrawn,
             -- which is exactly what leave_cancel refuses to do.
             WHERE leave_requests.leave_status = 'pending'
           RETURNING leave_id"#,
    )
    .bind(employee_id)
    .bind(date)
    .bind(kind)
    .bind(body.leave_reason.as_deref().unwrap_or("").trim())
    .bind(&requester)
    .fetch_one(&pool.db)
    .await;

    match inserted {
        Ok(r) => {
            let id: i32 = r.get("leave_id");
            GenericResponse::<i32>::single(id, "OK".to_string()).into_response()
        }
        // ON CONFLICT ... WHERE left the decided row untouched, so the upsert
        // returns no row: that date is already decided.
        Err(sqlx::Error::RowNotFound) => NoDataResponse::coded(
            "leave.decidedLocked",
            "Decided requests cannot be edited".to_string(),
            409,
        )
        .into_response(),
        Err(e) => {
            log::error!("{}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

/// Approve or reject a request.
///
/// Rejecting is modelled as `leave_status = 'rejected'` rather than a delete,
/// so the request and the decision survive for the audit trail.
#[post("/leave/decision")]
pub async fn leave_decision(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverLeaveDecision>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer.clone(), "leave", Action::Edit).await {
        return resp;
    }

    let status = if body.leave_approve { "approved" } else { "rejected" };
    let decider = get_session_user(pool.db.clone(), bearer).await.unwrap_or_default();

    let updated = sqlx::query(
        r#"UPDATE leave_requests
           SET leave_status = $1,
               decided_by = $2,
               update_at = EXTRACT(EPOCH FROM NOW())::BIGINT
           WHERE leave_id = $3 AND leave_status = 'pending'"#,
    )
    .bind(status)
    .bind(&decider)
    .bind(body.leave_id)
    .execute(&pool.db)
    .await;

    match updated {
        Ok(r) if r.rows_affected() == 1 => NoDataResponse::ok("OK".to_string()).into_response(),
        // No row: either the request was cancelled in another tab, or it is
        // already decided and must not be flipped again.
        Ok(_) => {
            let status: Option<String> =
                sqlx::query_scalar("SELECT leave_status FROM leave_requests WHERE leave_id = $1")
                    .bind(body.leave_id)
                    .fetch_optional(&pool.db)
                    .await
                    .unwrap_or(None);
            match status {
                Some(_) => NoDataResponse::coded(
                    "leave.decidedLocked",
                    "This request was already decided".to_string(),
                    409,
                )
                .into_response(),
                None => NoDataResponse::new("That request no longer exists".to_string(), 404)
                    .into_response(),
            }
        }
        Err(e) => {
            log::error!("{}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

#[post("/leave/cancel")]
pub async fn leave_cancel(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverLeaveCancel>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "leave", Action::Delete).await {
        return resp;
    }

    // Cancel is a hard delete rather than a status: the row records the fact that
    // someone asked and withdrew, which no report reads, and keeping it would
    // occupy the (employee, date) slot the unique constraint protects.
    // Only a pending request can be withdrawn: an approved or rejected one is a
    // decision record, and deleting it would let that day return to attendance.
    let deleted = sqlx::query(
        "DELETE FROM leave_requests WHERE leave_id = $1 AND leave_status = 'pending'",
    )
    .bind(body.leave_id)
    .execute(&pool.db)
    .await;

    match deleted {
        Ok(r) if r.rows_affected() == 1 => NoDataResponse::ok("OK".to_string()).into_response(),
        Ok(_) => {
            let status: Option<String> = sqlx::query_scalar(
                "SELECT leave_status FROM leave_requests WHERE leave_id = $1",
            )
            .bind(body.leave_id)
            .fetch_optional(&pool.db)
            .await
            .unwrap_or(None);
            match status {
                Some(_) => NoDataResponse::coded(
                    "leave.decidedLocked",
                    "Decided requests cannot be withdrawn".to_string(),
                    409,
                )
                .into_response(),
                None => NoDataResponse::new("That request no longer exists".to_string(), 404)
                    .into_response(),
            }
        }
        Err(e) => {
            log::error!("{}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}