use crate::{
    receiver::r_cdata::*,
    utility::stor::{AppState},
    models::m_command::Command,
};
use actix_web::{get, web::{self}, HttpResponse, Responder};
use chrono::Utc;

#[get("/getrequest")]
pub async fn get_request(data: web::Query<GetCData>, pool: web::Data<AppState>) -> impl Responder {
    // Update online status
    match sqlx::query(r#"UPDATE devices SET device_online = $1, device_state=$2 WHERE device_sn = $3"#)
        .bind(Utc::now().timestamp())
        .bind(1)
        .bind(data.sn.as_deref().unwrap_or(""))
        .execute(&pool.db)
        .await
        {
            Ok(_)=>(),
            Err(_)=>()
        };

    println!("GET REQUEST: {:?}", data.info.as_deref().unwrap_or("No Info"));
    
    match sqlx::query_as::<_, Command>(r#"SELECT * FROM commands WHERE command_destination=$1"#)
    .bind(data.sn.as_deref().unwrap_or(""))
    .fetch_all(&pool.db)
    .await
    {
        Ok(query)=>{
            if query.len() < 1 {
                return HttpResponse::Ok().content_type("text/plain").body("OK");
            }

            let mut commands: String = "".to_owned();
            for cmd in query {
                commands.push_str(&format!("C:{}:{}\n", cmd.command_id, cmd.command_params));
            }

            return HttpResponse::Ok().content_type("text/plain").body(commands);
        },
        Err(_)=>{
            return HttpResponse::Ok().content_type("text/plain").body("OK");
        }
    };
}