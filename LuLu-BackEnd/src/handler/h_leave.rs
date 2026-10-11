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

const KINDS: [&str; 3] = ["cuti", "izin", "sakit"];

fn parse_date(raw: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").ok()
}

#[get("/leave/list")]
pub async fn leave_list(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer.clone(), "leave", Action::View).await {
        return resp;
    }

    let from = query.get("from").and_then(|v| parse_date(v));
    let to = query.get("to").and_then(|v| parse_date(v));
    let employee = query
        .get("employee_id")
        .and_then(|v| v.trim().parse::<i32>().ok())
        .filter(|id| *id > 0);

    let rows = sqlx::query(
        r#"SELECT l.leave_id, l.leave_employee_id, l.leave_date, l.leave_kind,
                  l.leave_reason, l.leave_status, l.leave_source,
                  l.request_by, l.decided_by,
                  COALESCE(e.employee_fname, '') AS employee_fname,
                  COALESCE(e.employee_lname, '') AS employee_lname
           FROM leave_requests l
           LEFT JOIN employee e ON e.employee_id = l.leave_employee_id
           WHERE ($1::DATE IS NULL OR l.leave_date >= $1)
             AND ($2::DATE IS NULL OR l.leave_date <= $2)
             AND ($3::INTEGER IS NULL OR l.leave_employee_id = $3)
           ORDER BY l.leave_date DESC, l.leave_id DESC"#,
    )
    .bind(from)
    .bind(to)
    .bind(employee)
    .fetch_all(&pool.db)
    .await;

    match rows {
        Ok(rs) => {

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
            GenericResponse::<LeaveListRow>::ok(data, "OK".to_string()).into_response()
        }

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

    let rows = sqlx::query(
        r#"SELECT employee_id AS id,
                  COALESCE(employee_fname, '') AS first_name,
                  COALESCE(employee_lname, '') AS last_name,
                  COALESCE(employee_departement, '') AS departement
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
                    departement: r.get("departement"),
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
    departement: String,
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

    let requester = get_session_user(pool.db.clone(), bearer).await.unwrap_or_default();

    let inserted = sqlx::query(
        r#"INSERT INTO leave_requests
             (leave_employee_id, leave_date, leave_kind, leave_reason,
              leave_status, leave_source, request_by)
           VALUES ($1, $2, $3, $4, 'pending', 'manual', $5)
           ON CONFLICT (leave_employee_id, leave_date) DO UPDATE
             SET leave_kind = EXCLUDED.leave_kind,
                 leave_reason = EXCLUDED.leave_reason,
                 -- Editing a request returns it to pending: the old decision was
                 -- made on the old details and no longer applies.
                 leave_status = 'pending',
                 decided_by = '',
                 update_at = EXTRACT(EPOCH FROM NOW())::BIGINT
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
        Err(e) => {
            log::error!("{}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

/// Approve or reject a request.
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
           WHERE leave_id = $3"#,
    )
    .bind(status)
    .bind(&decider)
    .bind(body.leave_id)
    .execute(&pool.db)
    .await;

    match updated {
        Ok(r) if r.rows_affected() == 1 => NoDataResponse::ok("OK".to_string()).into_response(),
        // No row means the id belongs to a request that was cancelled in another
        // tab; saying so beats reporting success on nothing.
        Ok(_) => NoDataResponse::new("That request no longer exists".to_string(), 404).into_response(),
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
    let deleted = sqlx::query("DELETE FROM leave_requests WHERE leave_id = $1")
        .bind(body.leave_id)
        .execute(&pool.db)
        .await;

    match deleted {
        Ok(r) if r.rows_affected() == 1 => NoDataResponse::ok("OK".to_string()).into_response(),
        Ok(_) => NoDataResponse::new("That request no longer exists".to_string(), 404).into_response(),
        Err(e) => {
            log::error!("{}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}