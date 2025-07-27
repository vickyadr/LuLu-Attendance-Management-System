use crate::{
    models::m_shift::*, receiver::r_shift::ReceiverShift, utility::{authorization::is_login, stor::{AppState, GenericResponse, KeyValResponse, NoDataResponse}}
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

    match sqlx::query_as::<_, Shifts>(r#"SELECT * FROM shifts ORDER BY shift_id"#)
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => return HttpResponse::Ok().json(GenericResponse::<Shifts>::ok(
                                data,
                                format!("OK")
                            )),
        Err(_) => return HttpResponse::Ok().json(NoDataResponse::ok (
                                format!("No shift"),
                            ))
    }
        
}

#[post("/shift/add")]
pub async fn shift_add(pool: web::Data<AppState>, body: web::Json<ReceiverShift>, bearer: Option<ReqData<String>>) -> impl Responder {
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


    match sqlx::query(r#"INSERT INTO shifts (shift_name, shift_start_time, shift_end_time, shift_start_enroll, shift_end_enroll, shift_passday) VALUES ($1, $2, $3, $4, $5, $6)"#)
                .bind(body.name.clone())
                .bind(body.start_time)
                .bind(body.end_time)
                .bind(start_enroll)
                .bind(end_enroll)
                .bind(passday)
                .execute(&pool.db)
                .await
        {
            Ok(_data)=>{
                return HttpResponse::Ok().json(NoDataResponse::ok(
                    format!("Ok !!!"),
                ));
            },
            Err(e) => {
                println!("{}", e);
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Internal error, please try again!"),
                    500
                ));
            }
        }
}