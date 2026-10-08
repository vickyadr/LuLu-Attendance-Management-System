use crate::{
    models::m_command::Command,
    receiver::r_cdata::*,
    utility::{stor::AppState, wshub::WsOut},
};
use actix_web::{
    HttpResponse, Responder, get,
    web::{self},
};
use chrono::Utc;

#[get("/getrequest")]
pub async fn get_request(data: web::Query<GetCData>, pool: web::Data<AppState>) -> impl Responder {
    let sn = data.sn.as_deref().unwrap_or("");

    let prev_state: Option<i32> = sqlx::query_scalar::<_, Option<i32>>(
        r#"SELECT device_state FROM devices WHERE device_sn = $1"#,
    )
    .bind(sn)
    .fetch_optional(&pool.db)
    .await
    .ok()
    .flatten()
    .flatten();

    // Update online status.
    let touched = sqlx::query(
        r#"UPDATE devices SET device_online = $1, device_state = $2 WHERE device_sn = $3 AND device_state <> 3"#,
    )
    .bind(Utc::now().timestamp())
    .bind(1)
    .bind(sn)
    .execute(&pool.db)
    .await
    .map(|res| res.rows_affected())
    .unwrap_or(0);

    // Offline -> online is worth telling the dashboard about
    if touched > 0 && prev_state != Some(1) {
        let receivers = pool.ws.broadcast(WsOut {
            event: "device",
            message: "controller checked in".to_owned(),
            enroll_id: 0,
        });
        log::info!("ws broadcast device -> {receivers} client(s)");
    }

    log::info!(
        "GET REQUEST: {:?}",
        data.info.as_deref().unwrap_or("No Info")
    );

    match sqlx::query_as::<_, Command>(r#"SELECT * FROM commands WHERE command_destination=$1"#)
        .bind(data.sn.as_deref().unwrap_or(""))
        .fetch_all(&pool.db)
        .await
    {
        Ok(query) => {
            if query.is_empty() {
                return HttpResponse::Ok().content_type("text/plain").body("OK");
            }

            let mut commands: String = "".to_owned();
            for cmd in query {
                commands.push_str(&format!("C:{}:{}\n", cmd.command_id, cmd.command_params));
            }

            HttpResponse::Ok().content_type("text/plain").body(commands)
        }
        Err(_) => HttpResponse::Ok().content_type("text/plain").body("OK"),
    }
}
