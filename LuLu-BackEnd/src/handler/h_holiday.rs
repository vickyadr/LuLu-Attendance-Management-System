use crate::{
    models::m_holiday::*,
    utility::{
        authorization::*,
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

const KINDS: [&str; 3] = ["national", "collective", "office"];

/// Calendar days in `from`..=`to`, inclusive, as plain DATE values.
fn parse_date(raw: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").ok()
}

#[get("/holiday/list")]
pub async fn holiday_list(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "settings", Action::View).await {
        return resp;
    }

    // Both bounds are optional; an absent bound means "unbounded on that side",
    // so the settings panel can show the whole calendar and the report engine
    // can ask for exactly the window it is about to compute.
    let from = query.get("from").and_then(|v| parse_date(v));
    let to = query.get("to").and_then(|v| parse_date(v));

    let rows = sqlx::query(
        r#"SELECT holiday_date, holiday_name, holiday_kind, holiday_source, holiday_included
           FROM holiday
           WHERE ($1::DATE IS NULL OR holiday_date >= $1)
             AND ($2::DATE IS NULL OR holiday_date <= $2)
           ORDER BY holiday_date"#,
    )
    .bind(from)
    .bind(to)
    .fetch_all(&pool.db)
    .await;

    match rows {
        Ok(rs) => GenericResponse::<Holiday>::ok(
            rs.into_iter()
                .map(|r| Holiday {
                    holiday_date: r.get("holiday_date"),
                    holiday_name: r.get("holiday_name"),
                    holiday_kind: r.get("holiday_kind"),
                    holiday_source: r.get("holiday_source"),
            holiday_included: r.try_get("holiday_included").unwrap_or(false),
                })
                .collect(),
            "OK".to_string(),
        )
        .into_response(),

        // A missing table is a failed migration, and an empty calendar is the
        // normal state of a fresh install; neither is a client error, so the
        // report simply has no holidays to exclude.
        Err(e) => {
            log::error!("{}", e);
            GenericResponse::<Holiday>::ok(Vec::new(), "OK".to_string()).into_response()
        }
    }
}

#[post("/holiday/edit")]
pub async fn holiday_edit(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverHoliday>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "settings", Action::Edit).await {
        return resp;
    }

    let mut errs: HashMap<String, String> = HashMap::new();

    // A row is written only when at least one editable field arrives, so a
    // malformed request can never blank an existing entry's name.
    let Some(date_raw) = body.holiday_date.as_deref() else {
        return NoDataResponse::new("A date is required".to_string(), 400).into_response();
    };

    let Some(date) = parse_date(date_raw) else {
        errs.insert("holiday_date".to_string(), "Use the YYYY-MM-DD format".to_string());
        return KeyValResponse::<String, String>::new(
            errs,
            "There's was error while saving holiday".to_string(),
            403,
        )
        .into_response();
    };

    let kind = body.holiday_kind.as_deref().unwrap_or("national").to_string();
    if !KINDS.contains(&kind.as_str()) {
        errs.insert(
            "holiday_kind".to_string(),
            "Use one of national, collective, office".to_string(),
        );
    }

    let name = match body.holiday_name.as_deref().map(str::trim) {
        Some(n) if !n.is_empty() => n,
        _ => {
            errs.insert("holiday_name".to_string(), "A name is required".to_string());
            ""
        }
    };

    if !errs.is_empty() {
        return KeyValResponse::<String, String>::new(
            errs,
            "There's was error while saving holiday".to_string(),
            403,
        )
        .into_response();
    }

    // An import is only allowed to write provenance it actually owns; the
    // source column is operator-facing metadata, so a hand-written row keeps
    // 'manual' unless the caller explicitly says otherwise.
    let source = match body.holiday_source.as_deref() {
        Some("import") => "import",
        _ => "manual",
    };

    let done = sqlx::query(
        r#"INSERT INTO holiday (holiday_date, holiday_name, holiday_kind, holiday_source, holiday_included, updated_at)
           VALUES ($1, $2, $3, $4, COALESCE($5::BOOLEAN, FALSE), NOW())
           ON CONFLICT (holiday_date)
           DO UPDATE SET holiday_name = EXCLUDED.holiday_name,
                         holiday_kind = EXCLUDED.holiday_kind,
                         holiday_source = EXCLUDED.holiday_source,
                         -- $5, not EXCLUDED: a caller that omits the flag sends NULL, and
                         -- COALESCE then keeps whatever the operator already decided for
                         -- that date instead of silently resetting it to false.
                         holiday_included = COALESCE($5::BOOLEAN, holiday.holiday_included),
                         updated_at = NOW()"#,
    )
    .bind(date)
    .bind(name)
    .bind(&kind)
    .bind(source)
    .bind(body.holiday_included)
    .execute(&pool.db)
    .await;

    match done {
        Ok(_) => NoDataResponse::ok("Holiday saved !!!".to_string()).into_response(),
        Err(e) => {
            log::error!("{}", e);
            NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                .into_response()
        }
    }
}

#[post("/holiday/delete")]
pub async fn holiday_delete(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverHoliday>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "settings", Action::Delete).await {
        return resp;
    }

    let Some(date_raw) = body.holiday_date.as_deref() else {
        return NoDataResponse::new("A date is required".to_string(), 400).into_response();
    };

    let Some(date) = parse_date(date_raw) else {
        return KeyValResponse::<String, String>::new(
            HashMap::from([(
                "holiday_date".to_string(),
                "Use the YYYY-MM-DD format".to_string(),
            )]),
            "There's was error while deleting holiday".to_string(),
            403,
        )
        .into_response();
    };

    // DELETE is scoped to exactly one date. A wider delete would let a stray
    // request quietly turn the report's whole holiday window into working days.
    let done = sqlx::query("DELETE FROM holiday WHERE holiday_date = $1")
        .bind(date)
        .execute(&pool.db)
        .await;

    match done {
        Ok(r) if r.rows_affected() == 0 => {
            NoDataResponse::new("No holiday on that date".to_string(), 404).into_response()
        }
        Ok(_) => NoDataResponse::ok("Holiday deleted !!!".to_string()).into_response(),
        Err(e) => {
            log::error!("{}", e);
            NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                .into_response()
        }
    }
}

#[post("/holiday/import")]
pub async fn holiday_import(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverHolidayImport>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "settings", Action::Edit).await {
        return resp;
    }

    // Every row is validated before any row is written: a rejected import must
    // not leave the calendar half-populated, because a partial import looks
    // like a complete one to the operator who has not seen the error.
    let mut errs: HashMap<String, String> = HashMap::new();
    let mut clean: Vec<(NaiveDate, &str, &str)> = Vec::new();

    for (i, row) in body.holidays.iter().enumerate() {
        let Some(date) = parse_date(&row.holiday_date) else {
            errs.insert(
                format!("holidays[{}].holiday_date", i),
                "Use the YYYY-MM-DD format".to_string(),
            );
            continue;
        };
        let name = row.holiday_name.trim();
        if name.is_empty() {
            errs.insert(
                format!("holidays[{}].holiday_name", i),
                "A name is required".to_string(),
            );
            continue;
        }
        if !KINDS.contains(&row.holiday_kind.as_str()) {
            errs.insert(
                format!("holidays[{}].holiday_kind", i),
                "Use one of national, collective, office".to_string(),
            );
            continue;
        }
        clean.push((date, name, row.holiday_kind.as_str()));
    }

    if !errs.is_empty() {
        return KeyValResponse::<String, String>::new(
            errs,
            "There's was error while importing holidays".to_string(),
            403,
        )
        .into_response();
    }

    // Bounded so a malformed or hostile payload cannot pin the database for
    // minutes; two decades of holidays is far above any real calendar.
    if clean.len() > 5000 {
        return NoDataResponse::new("Too many rows, import 5000 or fewer".to_string(), 400)
            .into_response();
    }

    // Upsert rather than replace: an entry the operator added by hand for a
    // company closure keeps its name when a later import brings the same date.
    let mut tx = match pool.db.begin().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("{}", e);
            return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                .into_response();
        }
    };

    for (date, name, kind) in clean {
        if let Err(e) = sqlx::query(
            // holiday_included is deliberately absent from both the column list and the
            // DO UPDATE SET: a new date takes the column default (not counted), and a
            // date already in the table keeps whatever the operator decided for it.
            // A calendar feed knows the date and the name, never whether this office
            // pays differently that day, so it must not answer that question.
            r#"INSERT INTO holiday (holiday_date, holiday_name, holiday_kind, holiday_source, updated_at)
               VALUES ($1, $2, $3, 'import', NOW())
               ON CONFLICT (holiday_date)
               DO UPDATE SET holiday_name = EXCLUDED.holiday_name,
                             holiday_kind = EXCLUDED.holiday_kind,
                             holiday_source = 'import',
                             updated_at = NOW()"#,
        )
        .bind(date)
        .bind(name)
        .bind(kind)
        .execute(&mut *tx)
        .await
        {
            log::error!("{}", e);
            let _ = tx.rollback().await;
            return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                .into_response();
        }
    }

    if let Err(e) = tx.commit().await {
        log::error!("{}", e);
        return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
            .into_response();
    }

    NoDataResponse::ok("Holidays imported !!!".to_string()).into_response()
}
