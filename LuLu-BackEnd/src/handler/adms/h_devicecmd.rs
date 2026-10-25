use crate::{
    receiver::r_cdata::*,
    utility::stor::{AppState, KeyValResponse},
};
use actix_web::{
    HttpResponse, Responder, post,
    web::{self, Bytes},
};
use std::collections::HashMap;

#[post("/devicecmd")]
pub async fn post_device_cmd(
    pool: web::Data<AppState>,
    get: web::Query<ReceiverCData>,
    body: Bytes,
) -> impl Responder {
    let mut data: HashMap<String, String> = HashMap::new();
    // Lossy, not `unwrap`: this route is unauthenticated, so one stray non-UTF-8
    // byte from any caller would panic the worker instead of parsing the ASCII
    // device protocol this endpoint actually speaks.
    let data_body = String::from_utf8_lossy(&body).into_owned();
    let mut cmd_deleted: usize = 0;

    log::info!("POST CMD: {:?}", data_body);

    if get.sn.is_none() {
        data.insert("sn".to_string(), "Serial Number cant be empty".to_string());
    }

    if !data.is_empty() {
        return HttpResponse::BadRequest().json(KeyValResponse::<String, String>::new(
            data,
            "Post Error".to_string(),
            400,
        ));
    }

    if data_body.is_empty() {
        return HttpResponse::Ok().content_type("text/plain").body("");
    }

    // Delete commands from the database
    for dones in data_body.lines() {
        if dones.is_empty() {
            continue;
        }
        let done: Vec<&str> = dones.split('&').collect();
        let query = sqlx::query(r#"DELETE FROM commands WHERE command_id=$1"#)
            .bind(done[0].split('=').next_back().unwrap_or(""))
            .execute(&pool.db)
            .await
            .map_err(|e: sqlx::Error| e.to_string());

        if query.is_ok() {
            cmd_deleted += 1;
        }
    }

    if cmd_deleted == data_body.lines().count() {
        return HttpResponse::Ok().content_type("text/plain").body("OK");
    }

    HttpResponse::Ok().content_type("text/plain").body("")
}
