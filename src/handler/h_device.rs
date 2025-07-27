use std::collections::HashMap;

use crate::{
    models::m_device::*,
    receiver::r_device::*,
    utility::{
        authorization::is_login, 
        stor::{
            AppState, GenericResponse, NoDataResponse, KeyValResponse
        }
    }
};

use actix_web::{get, post, web::{self, ReqData}, HttpResponse, Responder};

#[get("/device/list")]
pub async fn device_list(pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
    
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


    match sqlx::query_as::<_, Devices>(r#"SELECT devices.device_name, devices.device_sn, devices.device_location, devices.device_timezone, devices.device_state, devices.device_online, handlers.handler_name FROM ( devices INNER JOIN handlers ON handlers.handler_id = devices.device_handler_id) ORDER BY devices.device_id DESC LIMIT 50"#)
    //.bind(bearer.into_inner())
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => {
            return HttpResponse::Ok().json(GenericResponse::<Devices>::ok(
                data,
                format!("ok"),
            ));
        },
        Err(e) => {
            println!("Error: {}", e);

            return HttpResponse::Ok().json(NoDataResponse::new(
                format!("Internal server error"),
                500
            ));                            
        }
    }
}

#[post("/device/add")]
pub async fn device_add(pool: web::Data<AppState>, body: web::Json<ReceiverDevice>, bearer: Option<ReqData<String>>) -> impl Responder {
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

    match body.sn.clone() {
        Some(o) => {
            if o.len().lt(&4) {
                data.insert("sn", format!("Too short"));
            }
        },
        None => {
            data.insert("sn", format!("Field required"));
        }
    }

    match body.location.clone() {
        Some(o) => {
            if o.len().ge(&160) {
                data.insert("location", format!("Max 160 character"));
            }
        },
        None => {
            data.insert("location", format!("Field required"));
        }
    }

    match body.handler.clone() {
        Some(_) => (),
        None => {
            data.insert("handler", format!("Field required"));
        }
    }

    if data.len() > 0 {
        return HttpResponse::Ok().json(KeyValResponse::<&str, String>::new(
            data,
            format!("There's was error while adding controller"),
            403
        ));
    }

    match sqlx::query(r#"INSERT INTO devices (device_name, device_sn, device_handler_id, device_location, device_timezone) VALUES ($1, $2, $3, $4, $5)"#)
                .bind(body.name.clone())
                .bind(body.sn.clone())
                .bind(body.handler)
                .bind(body.location.clone())
                .bind(body.timezone)
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
                let code =  e.as_database_error().unwrap().code().unwrap().into_owned();
                if code == "23505" {
                    return HttpResponse::Ok().json(NoDataResponse::new(
                        format!("This devices have registered!"),
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