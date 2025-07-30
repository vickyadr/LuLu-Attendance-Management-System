use crate::{
    models::m_shift::*, receiver::r_shift::*, utility::{authorization::is_login, stor::{AppState, GenericResponse, KeyValResponse, NoDataResponse}}
};
use actix_web::{get, post, web::{self, ReqData}, HttpResponse, Responder};
use std::collections::HashMap;

#[get("/shift/list")]
pub async fn shift_list(pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {

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

    match sqlx::query_as::<_, WorkTime>(r#"SELECT * FROM shifts ORDER BY shift_name"#)
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => return HttpResponse::Ok().json(GenericResponse::<WorkTime>::ok(
                                data,
                                format!("OK")
                            )),
        Err(_) => return HttpResponse::Ok().json(NoDataResponse::ok (
                                format!("No list"),
                            ))
    }
        
}

#[get("/shift/delete/{id}")]
pub async fn shift_delete(path: web::Path<i32>, pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
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

    match sqlx::query(r#"DELETE FROM shifts WHERE shift_id=$1"#)
    .bind(id)
    .execute(&pool.db)
    .await
    {
        Ok(_) => return HttpResponse::Ok().json(NoDataResponse::ok (
                                format!("Shift deleted !!!")
                            )),
        Err(_) => return HttpResponse::Ok().json(NoDataResponse::new (
                                format!("Shift not found"),
                                404
                            ))
    }
        
}

#[post("/shift/add")]
pub async fn shift_add(pool: web::Data<AppState>, body: web::Json<ReceiverWorkTIme>, bearer: Option<ReqData<String>>) -> impl Responder {
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

    match body.start_time.clone() {
        Some(o) => {
            if o.gt(&86400) {
                data.insert("start_time", format!("Use 00:xx instead"));
            }
        },
        None => {
            data.insert("start_time", format!("Field required"));
        }
    }

    match body.end_time.clone() {
        Some(o) => {
            if o.gt(&86400) {
                data.insert("end_time", format!("Use 00:xx instead"));
            }
        },
        None => {
            data.insert("end_time", format!("Field required"));
        }
    }

    match body.start_enroll.clone() {
        Some(o) => {
            if o.gt(&86400) {
                data.insert("start_enroll", format!("Use 00:xx instead"));
            }
        },
        None => {
            data.insert("start_enroll", format!("Field required"));
        }
    }

    match body.end_enroll.clone() {
        Some(o) => {
            if o.gt(&86400) {
                data.insert("end_enroll", format!("Use 00:xx instead"));
            }
        },
        None => {
            data.insert("end_enroll", format!("Field required"));
        }
    }

    if data.len() > 0 {
        return HttpResponse::Ok().json(KeyValResponse::<&str, String>::new(
            data,
            format!("There's was error while adding shift"),
            403
        ));
    }
    
    let start_enroll = body.start_time.unwrap() - body.start_enroll.unwrap();
    let end_enroll = body.end_time.unwrap() + body.end_enroll.unwrap();

    let passday = if body.start_time > body.end_time { 1 }else{ 0 };


    match sqlx::query_as::<_, WorkTime>(r#"INSERT INTO shifts (shift_name, shift_start_time, shift_end_time, shift_start_enroll, shift_end_enroll, shift_passday) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#)
                .bind(body.name.clone())
                .bind(body.start_time)
                .bind(body.end_time)
                .bind(start_enroll)
                .bind(end_enroll)
                .bind(passday)
                .fetch_one(&pool.db)
                .await
        {
            Ok(data) => return HttpResponse::Ok().json(GenericResponse::<WorkTime>::single(
                                                data,
                                                format!("Shift added !!!")
                                            )),
            Err(e) => {
                println!("{}", e);
                let code =  e.as_database_error().unwrap().message();
                if code.contains("duplicate key value violates") {
                    return HttpResponse::Ok().json(NoDataResponse::new(
                        format!("This shift name has been use!"),
                        405
                    ));
                }
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Internal error, please try again!"),
                    500
                ));
            }
        }
}

#[post("/shift/edit")]
pub async fn shift_edit(pool: web::Data<AppState>, body: web::Json<ReceiverWorkTIme>, bearer: Option<ReqData<String>>) -> impl Responder {
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

    match body.id.clone() {
        Some(o) => {
            if o.lt(&1) {
                data.insert("id", format!("Id not found"));
            }
        },
        None => {
            data.insert("id", format!("Required"));
        }
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

    match body.start_time.clone() {
        Some(o) => {
            if o.gt(&86400) {
                data.insert("start_time", format!("Use 00:xx instead"));
            }
        },
        None => {
            data.insert("start_time", format!("Field required"));
        }
    }

    match body.end_time.clone() {
        Some(o) => {
            if o.gt(&86400) {
                data.insert("end_time", format!("Use 00:xx instead"));
            }
        },
        None => {
            data.insert("end_time", format!("Field required"));
        }
    }

    match body.start_enroll.clone() {
        Some(o) => {
            if o.gt(&86400) {
                data.insert("start_enroll", format!("Use 00:xx instead"));
            }
        },
        None => {
            data.insert("start_enroll", format!("Field required"));
        }
    }

    match body.end_enroll.clone() {
        Some(o) => {
            if o.gt(&86400) {
                data.insert("end_enroll", format!("Use 00:xx instead"));
            }
        },
        None => {
            data.insert("end_enroll", format!("Field required"));
        }
    }

    if data.len() > 0 {
        return HttpResponse::Ok().json(KeyValResponse::<&str, String>::new(
            data,
            format!("There's was error while edit shift"),
            403
        ));
    }
    
    let start_enroll = body.start_time.unwrap() - body.start_enroll.unwrap();
    let end_enroll = body.end_time.unwrap() + body.end_enroll.unwrap();

    let passday = if body.start_time > body.end_time { 1 }else{ 0 };


    match sqlx::query_as::<_, WorkTime>(r#"UPDATE shifts SET shift_name=$1, shift_start_time=$2, shift_end_time=$3, shift_start_enroll=$4, shift_end_enroll=$5, shift_passday=$6 WHERE shift_id=$7 RETURNING *"#)
                .bind(body.name.clone())
                .bind(body.start_time)
                .bind(body.end_time)
                .bind(start_enroll)
                .bind(end_enroll)
                .bind(passday)
                .bind(body.id)
                .fetch_one(&pool.db)
                .await
        {
            Ok(data) => return HttpResponse::Ok().json(GenericResponse::<WorkTime>::single(
                                    data,
                                    format!("Data shift has been update !")
                                )),
            Err(e) => {
                println!("{}", e);
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Internal error, please try again!"),
                    500
                ));
            }
        }
}