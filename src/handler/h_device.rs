use crate::{
    utility::{
        stor::{AppState, GenericResponse, NoDataResponse},
        authorization::is_login,
    },
    models::{m_device::*},
};
use actix_web::{get, web::{self, ReqData}, HttpResponse, Responder};

#[get("/device/list")]
pub async fn device_list(pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
    
    match bearer {
        Some(bearer) => {
            match is_login(pool.db.clone(), bearer.into_inner())
            .await
            {
                Some(_level) => {
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
                },
                _=> ()
            }
        },
        _ => ()
    }

    HttpResponse::Ok().json(NoDataResponse::new(
        format!("Session token invalid"),
        400
    ))
}