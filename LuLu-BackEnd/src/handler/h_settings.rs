use crate::{
    models::m_settings::*, utility::{
        authorization::is_login, stor::{AppState, GenericResponse, NoDataResponse, KeyValResponse}
    }
};
use actix_web::{get, post, web::{self, ReqData}, HttpResponse, Responder};
use sqlx::Row;
use std::collections::HashMap;

const DEFAULT_LATE_TOL: i64 = 60;
const DEFAULT_OVERTIME_TOL: i64 = 60;
const MAX_TOL: i64 = 3600;

#[get("/settings/list")]
pub async fn settings_list(pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
    match is_login(pool.db.clone(), bearer).await {
        Some(level) =>{
            if level.lt(&100) {
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Sorry, doesnt have permission"),
                    403
                ))
            }
        },
        None => return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Session token invalid"),
                    401
                ))
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

    // A missing table is a failed migration, not a client error — serve the defaults.
    let (late, overtime) = match row {
        Ok(r) => (r.get::<i64, _>("late_tol"), r.get::<i64, _>("overtime_tol")),
        Err(_) => (DEFAULT_LATE_TOL, DEFAULT_OVERTIME_TOL),
    };

    HttpResponse::Ok().json(GenericResponse::<Settings>::ok(
        vec![
            Settings { setting_key: "late_tolerance_sec".to_string(), setting_value: late },
            Settings { setting_key: "overtime_tolerance_sec".to_string(), setting_value: overtime },
        ],
        format!("OK")
    ))
}

#[post("/settings/edit")]
pub async fn settings_edit(pool: web::Data<AppState>, body: web::Json<ReceiverSettings>, bearer: Option<ReqData<String>>) -> impl Responder {
    let mut errs: HashMap<String, String> = HashMap::new();

    match is_login(pool.db.clone(), bearer).await {
        Some(level) =>{
            if level.lt(&100) {
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Sorry, doesnt have permission"),
                    403
                ))
            }
        },
        None => return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Session token invalid"),
                    401
                ))
    }

    // Validate both fields before writing either, so a rejected value never half-applies.
    for (field, value) in [
        ("late_tolerance_sec", body.late_tolerance_sec),
        ("overtime_tolerance_sec", body.overtime_tolerance_sec),
    ] {
        if let Some(v) = value {
            if v < 0 || v > MAX_TOL {
                errs.insert(field.to_string(), format!("Use 0-3600 seconds"));
            }
        }
    }

    if errs.len() > 0 {
        return HttpResponse::Ok().json(KeyValResponse::<String, String>::new(
            errs,
            format!("There's was error while saving settings"),
            403
        ));
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
                   DO UPDATE SET setting_value = EXCLUDED.setting_value, updated_at = NOW()"#
            )
            .bind(key)
            .bind(v)
            .execute(&pool.db)
            .await;

            if let Err(e) = done {
                println!("{}", e);
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Internal error, please try again!"),
                    500
                ));
            }
        }
    }

    HttpResponse::Ok().json(NoDataResponse::ok(format!("Settings saved !!!")))
}
