use crate::{
    receiver::r_list::ReceiverList,
    utility::stor::{AppState, GenericResponse, KeyValResponse},
};
use actix_web::{get, post, web, HttpResponse, Responder};
use std::collections::HashMap;

use crate::models::m_content::Content;

#[get("/list")]
pub async fn get_list(pool: web::Data<AppState>) -> impl Responder {
    let query = sqlx::query_as::<_, Content>(r#"SELECT * FROM contents"#)
        .fetch_all(&pool.db)
        .await;

    if query.is_ok() {
        return HttpResponse::Ok().json(GenericResponse::<Content>::new(
            query.unwrap(),
            "Content List".to_string(),
        ));
    }

    HttpResponse::InternalServerError().json(GenericResponse::<i8>::new(
        Vec::new(),
        "No Content".to_string(),
    ))
}

#[post("/list")]
pub async fn post_list(pool: web::Data<AppState>, body: web::Json<ReceiverList>) -> impl Responder {
    let mut data: HashMap<String, String> = HashMap::new();

    if body.title.is_empty() {
        data.insert("title".to_string(), "Title cant be empty".to_string());
    }

    if body.sort.lt(&0) {
        data.insert("short".to_string(), "Please select short type".to_string());
    }

    if data.len() > 0 {
        return HttpResponse::InternalServerError().json(KeyValResponse::<String, String>::new(
            data,
            "Post Error".to_string(),
        ));
    }

    use chrono::{DateTime, TimeZone, Utc};
    let dt: DateTime<Utc> = Utc.with_ymd_and_hms(2015, 5, 15, 0, 0, 0).unwrap();

    let query =
            sqlx::query(r#"INSERT INTO contents (content_title, content_link, content_short, content_parrent, content_sub, content_create_at) VALUES (?, ?, ?, ?, ?, ?)"#)
                .bind(body.title.to_string())
                .bind(body.link.to_owned().unwrap_or("".to_string()))
                .bind(body.sort)
                .bind(body.parrent.to_owned().unwrap_or(0))
                .bind(body.sub.to_owned().unwrap_or(0))
                .bind(dt.timestamp())
                .execute(&pool.db)
                .await
                .map_err(|e: sqlx::Error| e.to_string());

    if let Err(_e) = query {
        //if e.contains("Duplicate entry") {
        //    return HttpResponse::BadRequest();
        //}

        return HttpResponse::InternalServerError().json(GenericResponse::<i8>::new(
            Vec::new(),
            "Insert data error".to_string(),
        ));
    }

    if query.is_ok() {
        return HttpResponse::Ok().json(GenericResponse::<i8>::new(
            Vec::new(),
            "Insert Success".to_string(),
        ));
    }

    return HttpResponse::InternalServerError().json(GenericResponse::<i8>::new(
        Vec::new(),
        "Internal Error".to_string(),
    ));
}
