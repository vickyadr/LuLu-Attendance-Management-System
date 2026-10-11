use crate::{
    models::m_settings::*,
    utility::{
        authorization::{Action, check_action_access},
        stor::{AppState, GenericResponse, IntoResponse, KeyValResponse, NoDataResponse},
    },
};
use actix_web::{
    Responder, get, post,
    web::{self, ReqData},
};
use sqlx::Row;
use std::collections::HashMap;

const DEFAULT_LATE_TOL: i64 = 60;
const DEFAULT_OVERTIME_TOL: i64 = 60;
const MAX_TOL: i64 = 3600;

#[get("/settings/list")]
pub async fn settings_list(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "settings", Action::View).await {
        return resp;
    }

    // Resolve both keys in one pass, falling back to the built-in default when the table is
    // absent or a row is missing, so the client always receives a complete settings object.
    let row = sqlx::query(
        r#"SELECT
             COALESCE(MAX(setting_value) FILTER (WHERE setting_key='late_tolerance_sec'), $1::BIGINT) AS late_tol,
             COALESCE(MAX(setting_value) FILTER (WHERE setting_key='overtime_tolerance_sec'), $2::BIGINT) AS overtime_tol
           FROM settings"#
    )
    .bind(DEFAULT_LATE_TOL)
    .bind(DEFAULT_OVERTIME_TOL)
    .fetch_one(&pool.db)
    .await;

    // A missing table is a failed migration, not a client error - serve the defaults.
    let (late, overtime) = match row {
        Ok(r) => (r.get::<i64, _>("late_tol"), r.get::<i64, _>("overtime_tol")),
        Err(_) => (DEFAULT_LATE_TOL, DEFAULT_OVERTIME_TOL),
    };

    GenericResponse::<Settings>::ok(
        vec![
            Settings {
                setting_key: "late_tolerance_sec".to_string(),
                setting_value: late,
            },
            Settings {
                setting_key: "overtime_tolerance_sec".to_string(),
                setting_value: overtime,
            },
        ],
        "OK".to_string(),
    )
    .into_response()
}

#[post("/settings/edit")]
pub async fn settings_edit(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverSettings>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let mut errs: HashMap<String, String> = HashMap::new();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "settings", Action::Edit).await {
        return resp;
    }

    // Validate both fields before writing either, so a rejected value never half-applies.
    for (field, value) in [
        ("late_tolerance_sec", body.late_tolerance_sec),
        ("overtime_tolerance_sec", body.overtime_tolerance_sec),
    ] {
        if let Some(v) = value
            && (!(0..=MAX_TOL).contains(&v))
        {
            errs.insert(field.to_string(), "Use 0-3600 seconds".to_string());
        }
    }

    if !errs.is_empty() {
        return KeyValResponse::<String, String>::new(
            errs,
            "There's was error while saving settings".to_string(),
            403,
        )
        .into_response();
    }

    for (key, value) in [
        ("late_tolerance_sec", body.late_tolerance_sec),
        ("overtime_tolerance_sec", body.overtime_tolerance_sec),
    ] {
        if let Some(v) = value {
            let done = sqlx::query(
                r#"INSERT INTO settings (setting_key, setting_value, updated_at)
                   VALUES ($1, $2, NOW())
                   ON CONFLICT (setting_key)
                   DO UPDATE SET setting_value = EXCLUDED.setting_value, updated_at = NOW()"#,
            )
            .bind(key)
            .bind(v)
            .execute(&pool.db)
            .await;

            if let Err(e) = done {
                log::error!("{}", e);
                return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                    .into_response();
            }
        }
    }

    NoDataResponse::ok("Settings saved !!!".to_string()).into_response()
}
