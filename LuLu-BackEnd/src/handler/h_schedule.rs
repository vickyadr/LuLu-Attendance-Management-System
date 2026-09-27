use crate::{
    models::m_schedule::*, receiver::r_schedule::*, utility::{authorization::is_login, stor::{AppState, GenericResponse, KeyValResponse, NoDataResponse}}
};
use actix_web::{get, post, web::{self, ReqData}, HttpResponse, Responder};
use std::collections::HashMap;

#[get("/schedule/list")]
pub async fn schedule_list(pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {

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

    match sqlx::query_as::<_, Schedules>(r#"SELECT schedules.schedule_id as schedule_id, schedules.schedule_name, schedules.schedule_shift_id, COALESCE(shifts.shift_name,'Libur') as shift_name, COALESCE(shifts.shift_start_time,0) as shift_start_time, COALESCE(shifts.shift_end_time,0) as shift_end_time, schedules.schedule_type, schedules.schedule_dom, schedules.schedule_parrent FROM schedules LEFT JOIN shifts ON shifts.shift_id = schedules.schedule_shift_id ORDER BY schedule_parrent, schedule_dom"#)
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => return HttpResponse::Ok().json(GenericResponse::<Schedules>::ok(
                                data,
                                format!("OK")
                            )),
        Err(e) =>
        {
            println!("E: {:?}", e);
            return HttpResponse::Ok().json(NoDataResponse::ok (
                                format!("Internal server error"),
                            ))
        }
    }
        
}

#[get("/schedule/delete/{id}")]
pub async fn schedule_delete(path: web::Path<i32>, pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
    let id = path.into_inner();
    
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

    match sqlx::query(r#"DELETE FROM schedules WHERE schedule_parrent=$1"#)
    .bind(id)
    .execute(&pool.db)
    .await
    {
        Ok(_) => return HttpResponse::Ok().json(NoDataResponse::ok (
                                format!("Schedule deleted !!!")
                            )),
        Err(_) => return HttpResponse::Ok().json(NoDataResponse::new (
                                format!("Schedule not found"),
                                404
                            ))
    }
        
}

#[post("/schedule/add")]
pub async fn schedule_add(pool: web::Data<AppState>, body: web::Json<ReceiverSchedule>, bearer: Option<ReqData<String>>) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();
    
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

    match body.name.clone() {
        Some(o) => {
            if o.len().lt(&3) {
                data.insert("name", format!("Too short"));
            }
        },
        None => {
            data.insert("name", format!("Field required"));
        }
    }

    match body.shift_id.clone() {
        Some(o) => {
            if o.len().lt(&1) {
                data.insert("shift", format!("Please select"));
            }
        },
        None => {
            data.insert("shift", format!("Field required"));
        }
    }
    
    match body.pattern.clone() {
        Some(v) => {
            // pattern is schedule_type: 1,7,14,21,28
            let expected = match v {
                1 => 1,
                7 => 7,
                14 => 14,
                21 => 21,
                28 => 28,
                0 => 1, // legacy flat 0 -> 1
                2 => 14, 3 => 21, 4 => 28, // legacy 2/3/4 compat
                _ => { data.insert("pattern", format!("Invalid pattern")); 0 }
            };
            if expected != 0 {
                if let Some(shifts) = body.shift_id.clone() {
                    if shifts.len() != expected as usize {
                        data.insert("shift", format!("Expected {} shifts for this pattern, got {}", expected, shifts.len()));
                    }
                }
            }
        },
        None => {
            data.insert("pattern", format!("Field required"));
        }
    }

    if data.len() > 0 {
        return HttpResponse::Ok().json(KeyValResponse::<&str, String>::new(
            data,
            format!("There's was error while adding schedules"),
            403
        ));
    }

    let raw_pattern = body.pattern.unwrap();
    let schedule_type: i32 = match raw_pattern {
        1 => 1,
        7 => 7,
        14 => 14,
        21 => 21,
        28 => 28,
        0 => 1,
        2 => 14, 3 => 21, 4 => 28,
        _ => 1,
    };
    let shifts = body.shift_id.clone().unwrap();
    println!("SCHEDULE_ADD name={:?} type={} shifts={:?}", body.name.clone().unwrap(), schedule_type, shifts);

    // Insert first row as parrent placeholder (dom 1) then fix parrent, then insert rest correctly with dom=i+1
    // Use transaction would be better, but keep simple sequential
    match sqlx::query_scalar::<_, i32>(r#"INSERT INTO schedules (schedule_name, schedule_shift_id, schedule_dom, schedule_parrent, schedule_type) VALUES ($1, $2, $3, $4, $5) RETURNING schedule_id"#)
                .bind(body.name.clone())
                .bind(shifts.get(0).unwrap())
                .bind(1)
                .bind(0)
                .bind(schedule_type)
                .fetch_one(&pool.db)
                .await
        {
            Ok(parrent_id) => {
                    println!("PARRENT : {:?}", parrent_id);
                match sqlx::query("UPDATE schedules SET schedule_parrent = $1 WHERE schedule_id = $1")
                    .bind(parrent_id)
                    .execute(&pool.db)
                    .await{
                        Ok(_)=>(),
                        Err(e)=> println!("ERROR : {:?}", e)
                    }
                
                // insert remaining doms 2..N
                for (idx, shift_id) in shifts.iter().enumerate() {
                    if idx == 0 { continue; }
                    let dom = (idx + 1) as i32;
                    match sqlx::query("INSERT INTO schedules (schedule_name, schedule_shift_id, schedule_dom, schedule_parrent, schedule_type) VALUES ($1, $2, $3, $4, $5)")
                    .bind(body.name.clone())
                    .bind(shift_id)
                    .bind(dom)
                    .bind(parrent_id)
                    .bind(schedule_type)
                    .execute(&pool.db)
                    .await{
                        Ok(_)=>(),
                        Err(e)=> println!("INSERT dom {} err {:?}", dom, e)
                    }
                }
                
                return HttpResponse::Ok().json(NoDataResponse::ok(
                    format!("Schedule added !!!")
                ));
            },
            Err(e) => {
                println!("{}", e);
                if let Some(db_err) = e.as_database_error() {
                    let code = db_err.message();
                    if code.contains("duplicate key value violates") {
                        return HttpResponse::Ok().json(NoDataResponse::new(
                            format!("This schedule name has been used!"),
                            405
                        ));
                    }
                    return HttpResponse::Ok().json(NoDataResponse::new(
                        format!("DB error: {}", code),
                        500
                    ));
                }
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Internal error, please try again!"),
                    500
                ));
            }
        }
}

#[post("/schedule/edit")]
pub async fn schedule_edit(pool: web::Data<AppState>, body: web::Json<ReceiverSchedule>, bearer: Option<ReqData<String>>) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();
    
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

    // Support two modes:
    // 1) Drag-drop single-row update: { id: schedule_id, shift: [new_shift_id] }
    // 2) Legacy name edit (not used) -> validate
    match body.id.clone() {
        Some(o) => {
            if o.lt(&1) {
                data.insert("id", format!("Id tidak valid"));
            }
        },
        None => {
            data.insert("id", format!("ID jadwal wajib"));
        }
    }

    // If shift is provided as single element, do quick shift_id update for that row
    if let Some(shifts) = body.shift_id.clone() {
        if shifts.len() == 1 {
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
                    return HttpResponse::Ok().json(NoDataResponse::ok(format!("Shift jadwal diperbarui")));
                },
                Ok(_) => {
                    return HttpResponse::Ok().json(NoDataResponse::new(format!("Jadwal tidak ditemukan"), 404));
                },
                Err(e) => {
                    println!("schedule_edit shift update err: {:?}", e);
                    return HttpResponse::Ok().json(NoDataResponse::new(format!("Gagal update shift"), 500));
                }
            }
        }
    }

    // Fallback: if name + pattern + shift array provided, treat as full replace (delete & recreate) is not handled here
    // Just validate name for now
    if let Some(o) = body.name.clone() {
        if o.len() < 3 { data.insert("name", format!("Nama minimal 3 karakter")); }
    }

    if data.len() > 0 {
        return HttpResponse::Ok().json(KeyValResponse::<&str, String>::new(
            data,
            format!("Gagal edit jadwal"),
            403
        ));
    }

    return HttpResponse::Ok().json(NoDataResponse::new(format!("Format edit tidak didukung — kirim id + shift"), 400));
}