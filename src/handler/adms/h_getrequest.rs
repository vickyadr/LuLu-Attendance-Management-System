use crate::{
    receiver::r_cdata::*,
    utility::stor::{AppState},
    models::m_command::Command,
};
use actix_web::{get, web::{self}, HttpResponse, Responder};

#[get("/getrequest")]
pub async fn get_request(data: web::Query<GetCData>, pool: web::Data<AppState>) -> impl Responder {
    // Update online status
    /*let query = sqlx::query(r#"UPDATE devices SET device_online = NOW() WHERE device_sn = ?"#)
        .bind(data.sn.as_deref().unwrap_or(""))
        .execute(&pool.db)
        .await;*/
    println!("GET REQUEST: {:?}", data.info.as_deref().unwrap_or("No Info"));

    let query = sqlx::query_as::<_, Command>(r#"SELECT * FROM commands WHERE command_destination=$1"#)
    .bind(data.sn.as_deref().unwrap_or(""))
    .fetch_all(&pool.db)
    .await
    .map_err(|e: sqlx::Error| e.to_string())
    .expect("Failed to fetch commands");

    if !query.is_empty() {
        let mut commands: String = "".to_owned();

        for cmd in query {
            commands.push_str(&format!("C:{}:{}\n", cmd.command_id, cmd.command_params));
            //C:112:DATA QUERY USERINFO PIN=99999
        }

        return HttpResponse::Ok().content_type("text/plain").body(commands);
    }

    return HttpResponse::Ok().content_type("text/plain").body("OK");
}
