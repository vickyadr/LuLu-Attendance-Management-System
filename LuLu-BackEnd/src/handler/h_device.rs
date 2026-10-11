use std::collections::HashMap;

use crate::{
    models::m_device::*,
    receiver::r_device::*,
    service::pull_scheduler::{cancel_pull_task, spawn_pull_task, PullRegistry},
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

    match sqlx::query_as::<_, Devices>(r#"SELECT devices.device_handler_id, devices.device_id, devices.device_name, devices.device_sn, devices.device_location, devices.device_timezone, devices.device_state, devices.device_online, devices.device_ip, handlers.handler_name FROM ( devices INNER JOIN handlers ON handlers.handler_id = devices.device_handler_id) ORDER BY devices.device_id ASC LIMIT 50"#)
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
    registry: web::Data<PullRegistry>,
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

    if let Some(ref ip) = body.ip {
        if !ip.is_empty() && ip.parse::<std::net::IpAddr>().is_err() {
            data.insert("ip", "Not a valid IP address".to_string());
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
    match sqlx::query_as::<_, Devices>(r#"WITH query AS (INSERT INTO devices (device_name, device_sn, device_handler_id, device_location, device_timezone, device_state, device_ip) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *) SELECT device_handler_id, device_id, device_name, device_sn, device_location, device_timezone, device_state, device_online, device_ip, handlers.handler_name FROM query INNER JOIN handlers ON handlers.handler_id = device_handler_id"#)
                .bind(body.name.clone())
                .bind(body.sn.clone())
                .bind(body.handler)
                .bind(body.location.clone())
                .bind(body.timezone)
                .bind(2)
                .bind(body.ip.clone())
                .fetch_one(&pool.db)
                .await
        {
            Ok(data) => {

            if data.device_handler_id == 5 {
                spawn_pull_task(data.device_id, pool.clone(), registry.clone()).await;
            }

            GenericResponse::<Devices>::single(
                                    data,
                                    "New devices added !!!".to_string()
                                ).into_response()
        },
            Err(e) => {
                log::error!("{}", e);

                if let Some(db_err) = e.as_database_error() {
                    if db_err.message().contains("duplicate key value violates") {
                        return NoDataResponse::new(
                            "This devices have registered!".to_string(),
                            405
                        ).into_response();
                    }
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
    registry: web::Data<PullRegistry>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let id = path.into_inner();

    if let Ok(resp) =
        check_action_access(pool.db.clone(), bearer, "controller", Action::Delete).await
    {
        return resp;
    }

    // Cancel any active pull task before deleting.
    cancel_pull_task(&registry, id).await;

    match sqlx::query(r#"DELETE FROM devices WHERE device_id=$1"#)
        .bind(id)
        .execute(&pool.db)
        .await
    {
        Ok(_) => NoDataResponse::ok("Device deleted".to_string()).into_response(),
        Err(_) => NoDataResponse::new("Device not found".to_string(), 404).into_response(),
    }
}

#[get("/device/disable/{id}")]
pub async fn device_disable(
    path: web::Path<i32>,
    pool: web::Data<AppState>,
    registry: web::Data<PullRegistry>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let id = path.into_inner();

    if let Ok(resp) =
        check_action_access(pool.db.clone(), bearer, "controller", Action::Edit).await
    {
        return resp;
    }

    let current: Option<i32> = match sqlx::query_scalar::<_, i32>(
        r#"SELECT device_state FROM devices WHERE device_id = $1"#,
    )
    .bind(id)
    .fetch_optional(&pool.db)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            log::error!("device {id} disable lookup failed: {e}");
            return NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                .into_response();
        }
    };

    let Some(current) = current else {
        return NoDataResponse::new("Device not found".to_string(), 404).into_response();
    };

    let new_state = if current == 3 { 2 } else { 3 };

    match sqlx::query(
        r#"UPDATE devices SET device_state = $1, update_at = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE device_id = $2"#,
    )
    .bind(new_state)
    .bind(id)
    .execute(&pool.db)
    .await
    {
        Ok(_) => {
            // A disabled device must not keep dialling the machine, so stop the
            // pull loop too; re-enabling restarts it.
            if new_state == 3 {
                cancel_pull_task(&registry, id).await;
            } else if let Ok(Some(5)) =
                sqlx::query_scalar::<_, i32>(
                    r#"SELECT device_handler_id FROM devices WHERE device_id = $1"#,
                )
                .bind(id)
                .fetch_optional(&pool.db)
                .await
            {
                spawn_pull_task(id, pool.clone(), registry.clone()).await;
            }

            let message = if new_state == 3 {
                format!("Device {id} disabled")
            } else {
                format!("Device {id} enabled")
            };
            NoDataResponse::ok(message).into_response()
        }
        Err(e) => {
            log::error!("device {id} disable update failed: {e}");
            NoDataResponse::new("Internal error, please try again!".to_string(), 500)
                .into_response()
        }
    }
}

#[post("/device/edit")]
pub async fn device_edit(
    pool: web::Data<AppState>,
    registry: web::Data<PullRegistry>,
    body: web::Json<ReceiverDevice>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    if let Ok(resp) = check_action_access(pool.db.clone(), bearer, "controller", Action::Edit).await
    {
        return resp;
    }


    if body.id.is_none() {
        data.insert("id", "Field required".to_string());
    }

    if let Some(ref n) = body.name {
        if n.len().lt(&3) {
            data.insert("name", "Too short".to_string());
        }
    }

    if let Some(ref o) = body.sn {
        if o.len().lt(&4) {
            data.insert("sn", "Too short".to_string());
        }
    }

    if let Some(ref o) = body.location {
        if o.len().ge(&160) {
            data.insert("location", "Max 160 character".to_string());
        }
    }

    // Empty means unset; the pull reader needs it to dial the device.
    // IPv6 included in the format, but the device speaks IPv4 over TCP 4370.
    if let Some(ref ip) = body.ip {
        if !ip.is_empty() && ip.parse::<std::net::IpAddr>().is_err() {
            data.insert("ip", "Not a valid IP address".to_string());
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
    match sqlx::query_as::<_, Devices>(r#"UPDATE devices SET device_name=COALESCE($1, device_name), device_handler_id=COALESCE($3, device_handler_id), device_location=COALESCE($4, device_location), device_timezone=COALESCE($5, device_timezone), device_ip=COALESCE($6, device_ip), update_at=EXTRACT(EPOCH FROM NOW())::BIGINT FROM handlers WHERE devices.device_handler_id = handlers.handler_id AND device_id=$2 RETURNING devices.device_id, devices.device_name, devices.device_sn, devices.device_handler_id, devices.device_location, devices.device_timezone, devices.device_state, devices.device_online, devices.device_ip, handlers.handler_name"#)
                .bind(body.name.clone())
                .bind(body.id)
                .bind(body.handler)
                .bind(body.location.clone())
                .bind(body.timezone)
                .bind(body.ip.clone())
                .fetch_one(&pool.db)
                .await
        {
            Ok(data) => {
            let still_zknet = data.device_handler_id == 5;
            if still_zknet && data.device_state != 3 {
                spawn_pull_task(data.device_id, pool.clone(), registry.clone()).await;
            } else {
                cancel_pull_task(&registry, data.device_id).await;
            }

            GenericResponse::<Devices>::single(
                                    data,
                                    "Device data updated !!!".to_string()
                                ).into_response()
        },
            Err(e) => {
                log::error!("{}", e);

                if let Some(db_err) = e.as_database_error() {
                    if db_err.message().contains("duplicate key value violates") {
                        return NoDataResponse::new(
                            "This devices have registered!".to_string(),
                            405
                        ).into_response();
                    }
                }
                NoDataResponse::new(
                    "Internal error, please try again!".to_string(),
                    500
                ).into_response()
            }
        }
}
