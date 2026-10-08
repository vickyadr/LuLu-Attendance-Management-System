use crate::{
    models::m_schedule::*,
    receiver::r_schedule::*,
    utility::{
        authorization::{Action, check_action_access},
        stor::{AppState, GenericResponse, IntoResponse, KeyValResponse, NoDataResponse},
    },
};
use actix_web::{
    Responder, get, post,
    web::{self, ReqData},
};
use std::collections::HashMap;

#[get("/schedule/list")]
pub async fn schedule_list(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "shift", Action::View).await {
        return resp;
    }

    match sqlx::query_as::<_, Schedules>(r#"SELECT schedules.schedule_id as schedule_id, schedules.schedule_name, schedules.schedule_shift_id, COALESCE(shifts.shift_name,'Libur') as shift_name, COALESCE(shifts.shift_start_time,0) as shift_start_time, COALESCE(shifts.shift_end_time,0) as shift_end_time, schedules.schedule_type, schedules.schedule_dom, schedules.schedule_parrent FROM schedules LEFT JOIN shifts ON shifts.shift_id = schedules.schedule_shift_id ORDER BY schedule_parrent, schedule_dom"#)
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => GenericResponse::<Schedules>::ok(
                                data,
                                "OK".to_string()
                            ).into_response(),
        Err(e) =>
        {
            log::info!("E: {:?}", e);
            NoDataResponse::ok (
                                "Internal server error".to_string(),
                            ).into_response()
        }
    }
}

#[get("/schedule/delete/{id}")]
pub async fn schedule_delete(
    path: web::Path<i32>,
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let id = path.into_inner();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "shift", Action::Delete).await {
        return resp;
    }

    match sqlx::query(r#"DELETE FROM schedules WHERE schedule_parrent=$1"#)
        .bind(id)
        .execute(&pool.db)
        .await
    {
        Ok(_) => NoDataResponse::ok("Schedule deleted !!!".to_string()).into_response(),
        Err(_) => NoDataResponse::new("Schedule not found".to_string(), 404).into_response(),
    }
}

#[post("/schedule/add")]
pub async fn schedule_add(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverSchedule>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "shift", Action::Add).await {
        return resp;
    }

    match body.name.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("name", "Too short".to_string());
            }
        }
        None => {
            data.insert("name", "Field required".to_string());
        }
    }

    match body.shift_id.clone() {
        Some(o) => {
            if o.len().lt(&1) {
                data.insert("shift", "Please select".to_string());
            }
        }
        None => {
            data.insert("shift", "Field required".to_string());
        }
    }

    match body.pattern {
        Some(v) => {
            // pattern is schedule_type: 1,7,14,21,28
            let expected = match v {
                1 => 1,
                7 => 7,
                14 => 14,
                21 => 21,
                28 => 28,
                0 => 1, // legacy flat 0 -> 1
                2 => 14,
                3 => 21,
                4 => 28, // legacy 2/3/4 compat
                _ => {
                    data.insert("pattern", "Invalid pattern".to_string());
                    0
                }
            };
            if expected != 0
                && let Some(shifts) = body.shift_id.clone()
                && shifts.len() != expected as usize
            {
                data.insert(
                    "shift",
                    format!(
                        "Expected {} shifts for this pattern, got {}",
                        expected,
                        shifts.len()
                    ),
                );
            }
        }
        None => {
            data.insert("pattern", "Field required".to_string());
        }
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(
            data,
            "There's was error while adding schedules".to_string(),
            403,
        )
        .into_response();
    }

    let raw_pattern = body.pattern.unwrap();
    let schedule_type: i32 = match raw_pattern {
        1 => 1,
        7 => 7,
        14 => 14,
        21 => 21,
        28 => 28,
        0 => 1,
        2 => 14,
        3 => 21,
        4 => 28,
        _ => 1,
    };
    let shifts = body.shift_id.clone().unwrap();
    log::info!(
        "SCHEDULE_ADD name={:?} type={} shifts={:?}",
        body.name.clone().unwrap(),
        schedule_type,
        shifts
    );


    let mut tx = match pool.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("schedule_add begin err: {:?}", e);
            return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                .into_response();
        }
    };

    let parrent_id = match sqlx::query_scalar::<_, i32>(r#"INSERT INTO schedules (schedule_name, schedule_shift_id, schedule_dom, schedule_parrent, schedule_type) VALUES ($1, $2, $3, $4, $5) RETURNING schedule_id"#)
                .bind(body.name.clone())
                .bind(shifts.first().unwrap())
                .bind(1)
                .bind(0)
                .bind(schedule_type)
                .fetch_one(&mut *tx)
                .await
        {
            Ok(id) => id,
            Err(e) => {
                log::error!("schedule parent insert err: {}", e);
                if let Some(db_err) = e.as_database_error() {
                    let code = db_err.message();
                    if code.contains("duplicate key value violates") {
                        return NoDataResponse::new(
                            "This schedule name has been used!".to_string(),
                            405
                        ).into_response();
                    }
                    return NoDataResponse::new(
                        format!("DB error: {}", code),
                        500
                    ).into_response();
                }
                return NoDataResponse::new(
                    "Internal error, please try again!".to_string(),
                    500
                ).into_response();
            }
        };

    log::info!("SCHEDULE_ADD parent_id={}", parrent_id);

    if let Err(e) = sqlx::query("UPDATE schedules SET schedule_parrent = $1 WHERE schedule_id = $1")
        .bind(parrent_id)
        .execute(&mut *tx)
        .await
    {
        log::error!("schedule parrent update err: {:?}", e);
        return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
            .into_response();
    }

    // insert remaining doms 2..N
    for (idx, shift_id) in shifts.iter().enumerate() {
        if idx == 0 {
            continue;
        }
        let dom = (idx + 1) as i32;
        if let Err(e) = sqlx::query("INSERT INTO schedules (schedule_name, schedule_shift_id, schedule_dom, schedule_parrent, schedule_type) VALUES ($1, $2, $3, $4, $5)")
            .bind(body.name.clone())
            .bind(shift_id)
            .bind(dom)
            .bind(parrent_id)
            .bind(schedule_type)
            .execute(&mut *tx)
            .await
        {
            log::error!("schedule dom {} insert err: {:?}", dom, e);

            return NoDataResponse::new(
                format!("Failed to save schedule row {}", dom),
                500
            ).into_response();
        }
    }

    if let Err(e) = tx.commit().await {
        log::error!("schedule_add commit err: {:?}", e);
        return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
            .into_response();
    }

    NoDataResponse::ok("Schedule added !!!".to_string()).into_response()
}

#[post("/schedule/edit")]
pub async fn schedule_edit(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverSchedule>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "shift", Action::Edit).await {
        return resp;
    }

    // Support two modes:
    // 1) Drag-drop single-row update: { id: schedule_id, shift: [new_shift_id] }
    // 2) Legacy name edit (not used) -> validate
    match body.id {
        Some(o) => {
            if o.lt(&1) {
                data.insert("id", "Id tidak valid".to_string());
            }
        }
        None => {
            data.insert("id", "ID jadwal wajib".to_string());
        }
    }

    // If shift is provided as single element, do quick shift_id update for that row
    if let Some(shifts) = body.shift_id.clone()
        && shifts.len() == 1
    {
        let id = body.id.unwrap();
        let new_shift = shifts[0];
        // allow 0 = Libur
        match sqlx::query(r#"UPDATE schedules SET schedule_shift_id=$1 WHERE schedule_id=$2"#)
            .bind(new_shift)
            .bind(id)
            .execute(&pool.db)
            .await
        {
            Ok(r) if r.rows_affected() > 0 => {
                return NoDataResponse::ok("Shift jadwal diperbarui".to_string()).into_response();
            }
            Ok(_) => {
                return NoDataResponse::new("Jadwal tidak ditemukan".to_string(), 404)
                    .into_response();
            }
            Err(e) => {
                log::error!("schedule_edit shift update err: {:?}", e);
                return NoDataResponse::new("Gagal update shift".to_string(), 500).into_response();
            }
        }
    }

    // Fallback: if name + pattern + shift array provided, treat as full replace (delete & recreate) is not handled here
    // Just validate name for now
    if let Some(o) = body.name.clone()
        && o.len() < 3
    {
        data.insert("name", "Nama minimal 3 karakter".to_string());
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(data, "Gagal edit jadwal".to_string(), 403)
            .into_response();
    }

    NoDataResponse::new(
        "Format edit tidak didukung — kirim id + shift".to_string(),
        400,
    )
    .into_response()
}
