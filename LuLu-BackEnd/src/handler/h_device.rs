use std::collections::HashMap;

use crate::{
    models::m_device::*,
    receiver::r_device::*,
    utility::{
        authorization::{Action, check_action_access},
        stor::{AppState, GenericResponse, IntoResponse, KeyValResponse, NoDataResponse},
    },
};

use actix_web::{
    Responder, get, post,
    web::{self, ReqData},
};

#[get("/device/list")]
pub async fn device_list(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "controller", Action::View).await
    {
        return resp;
    }

    match sqlx::query_as::<_, Devices>(r#"SELECT devices.device_handler_id, devices.device_id, devices.device_name, devices.device_sn, devices.device_location, devices.device_timezone, devices.device_state, devices.device_online, handlers.handler_name FROM ( devices INNER JOIN handlers ON handlers.handler_id = devices.device_handler_id) ORDER BY devices.device_id ASC LIMIT 50"#)
    //.bind(bearer.into_inner())
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => {
            GenericResponse::<Devices>::ok(
                data,
                "ok".to_string(),
            ).into_response()
        },
        Err(e) => {
            log::error!("Error: {}", e);

            NoDataResponse::new(
                "Internal server error".to_string(),
                500
            ).into_response()
        }
    }
}

#[post("/device/add")]
pub async fn device_add(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverDevice>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "controller", Action::Add).await
    {
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

    match body.sn.clone() {
        Some(o) => {
            if o.len().lt(&4) {
                data.insert("sn", "Too short".to_string());
            }
        }
        None => {
            data.insert("sn", "Field required".to_string());
        }
    }

    match body.location.clone() {
        Some(o) => {
            if o.len().ge(&160) {
                data.insert("location", "Max 160 character".to_string());
            }
        }
        None => {
            data.insert("location", "Field required".to_string());
        }
    }

    match body.handler {
        Some(_) => (),
        None => {
            data.insert("handler", "Field required".to_string());
        }
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(
            data,
            "There's was error while adding controller".to_string(),
            403,
        )
        .into_response();
    }
    //match sqlx::query_as::<_, Devices>(r#"SELECT devices.device_handler_id, devices.device_id, devices.device_name, devices.device_sn, devices.device_location, devices.device_timezone, devices.device_state, devices.device_online, handlers.handler_name FROM ( devices INNER JOIN handlers ON handlers.handler_id = devices.device_handler_id) ORDER BY devices.create_at DESC"#)
    match sqlx::query_as::<_, Devices>(r#"WITH query AS (INSERT INTO devices (device_name, device_sn, device_handler_id, device_location, device_timezone, device_state) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *) SELECT device_handler_id, device_id, device_name, device_sn, device_location, device_timezone, device_state, device_online, handlers.handler_name FROM query INNER JOIN handlers ON handlers.handler_id = device_handler_id"#)
                .bind(body.name.clone())
                .bind(body.sn.clone())
                .bind(body.handler)
                .bind(body.location.clone())
                .bind(body.timezone)
                .bind(2)
                .fetch_one(&pool.db)
                .await
        {
            Ok(data) => GenericResponse::<Devices>::single(
                                    data,
                                    "New devices added !!!".to_string()
                                ).into_response(),
            Err(e) => {
                log::error!("{}", e);
                let code =  e.as_database_error().unwrap().message();
                if code.contains("duplicate key value violates") {
                    return NoDataResponse::new(
                        "This devices have registered!".to_string(),
                        405
                    ).into_response();
                }
                NoDataResponse::new(
                    "Internal error, please try again!".to_string(),
                    500
                ).into_response()
            }
        }
}

#[get("/device/delete/{id}")]
pub async fn device_delete(
    path: web::Path<i32>,
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let id = path.into_inner();

    if let Ok(resp) =
        check_action_access(pool.db.clone(), bearer, "controller", Action::Delete).await
    {
        return resp;
    }

    match sqlx::query(r#"DELETE FROM devices WHERE device_id=$1"#)
        .bind(id)
        .execute(&pool.db)
        .await
    {
        Ok(_) => NoDataResponse::ok("Device deleted".to_string()).into_response(),
        Err(_) => NoDataResponse::new("Device not found".to_string(), 404).into_response(),
    }
}

#[post("/device/edit")]
pub async fn device_edit(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverDevice>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "controller", Action::Edit).await
    {
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

    match body.sn.clone() {
        Some(o) => {
            if o.len().lt(&4) {
                data.insert("sn", "Too short".to_string());
            }
        }
        None => {
            data.insert("sn", "Field required".to_string());
        }
    }

    match body.location.clone() {
        Some(o) => {
            if o.len().ge(&160) {
                data.insert("location", "Max 160 character".to_string());
            }
        }
        None => {
            data.insert("location", "Field required".to_string());
        }
    }

    match body.handler {
        Some(_) => (),
        None => {
            data.insert("handler", "Field required".to_string());
        }
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(
            data,
            "There's was error while adding controller".to_string(),
            403,
        )
        .into_response();
    }
    //match sqlx::query_as::<_, Devices>(r#"SELECT devices.device_handler_id, devices.device_id, devices.device_name, devices.device_sn, devices.device_location, devices.device_timezone, devices.device_state, devices.device_online, handlers.handler_name FROM ( devices INNER JOIN handlers ON handlers.handler_id = devices.device_handler_id) WHERE devices.device_id=$1"#)
    match sqlx::query_as::<_, Devices>(r#"UPDATE devices SET device_name=$1, device_handler_id=$3, device_location=$4, device_timezone=$5 FROM handlers WHERE devices.device_handler_id = handlers.handler_id AND device_id=$2 RETURNING *"#)
                .bind(body.name.clone())
                .bind(body.id)
                .bind(body.handler)
                .bind(body.location.clone())
                .bind(body.timezone)
                .fetch_one(&pool.db)
                .await
        {
            Ok(data) => GenericResponse::<Devices>::single(
                                    data,
                                    "Device data updated !!!".to_string()
                                ).into_response(),
            Err(e) => {
                log::error!("{}", e);
                let code =  e.as_database_error().unwrap().message();
                if code.contains("duplicate key value violates") {
                    return NoDataResponse::new(
                        "This devices have registered!".to_string(),
                        405
                    ).into_response();
                }
                NoDataResponse::new(
                    "Internal error, please try again!".to_string(),
                    500
                ).into_response()
            }
        }
}
