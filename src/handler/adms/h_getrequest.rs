use crate::{
    receiver::r_cdata::*,
    utility::stor::{AppState},
};
use actix_web::{get, web::{self}, HttpResponse, Responder};

#[get("/getrequest")]
pub async fn get_request(data: web::Query<GetCData>, _pool: web::Data<AppState>) -> impl Responder {
    // Update online status
    /*let query = sqlx::query(r#"UPDATE devices SET device_online = NOW() WHERE device_sn = ?"#)
        .bind(data.sn.as_deref().unwrap_or(""))
        .execute(&pool.db)
        .await;*/
    println!("GET REQUEST: {:?}", data.info.as_deref().unwrap_or("No Info"));
    return HttpResponse::Ok().content_type("text/plain").body("OK");
}
