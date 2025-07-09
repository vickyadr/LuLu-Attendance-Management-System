use crate::{
    receiver::r_login::ReceiverLogin,
    utility::stor::{AppState, GenericResponse, KeyValResponse, NoDataResponse},
    models::{m_login::*},
};
use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest};
//use actix_web_httpauth::{Auth, Bearer};
//use actix_hash::BodySha3_256;
use std::collections::HashMap;
use chrono::{Utc};

#[get("/login/user")]
pub async fn login_check(pool: web::Data<AppState>) -> impl Responder {
    let user_fname = sqlx::query(r#"SELECT users.user_fname FROM users INNER JOIN sessions ON sessions.session_user_id = users.user_id WHERE sessions.session_token = $1"#)
        .bind("bearer") // Bearer yet implemented
        .fetch_one(&pool.db)
        .await
        .map_err(|e|{println!("{}", e)});

    if user_fname.is_ok() {
        return HttpResponse::Ok().json(NoDataResponse::new(
            format!("OK"),
        ));
    }

    HttpResponse::InternalServerError().json(NoDataResponse::new(
        format!("Session token invalid"),
    ))
}

#[post("/login")]
pub async fn login_act(pool: web::Data<AppState>, body: web::Json<ReceiverLogin>, req: HttpRequest) -> impl Responder {
    let mut data: HashMap<String, String> = HashMap::new();
    if body.nickname.is_empty() {
        data.insert("nickname".to_string(), format!("Please enter your user id"));
    }else if body.nickname.len().lt(&3) {
        data.insert("nickname".to_string(), format!("User id too sort, are you sure?"));
    }

    if let Some(pass) = body.password.clone() {
        if pass.len().lt(&8) {
            data.insert("password".to_string(), format!("Password too sort, please think again"));
        }
    } else {
        data.insert("password".to_string(), format!("Please enter your password"));
    }

    if data.len() > 0 {
        return HttpResponse::BadRequest().json(KeyValResponse::<String, String>::new(
            data,
            format!("There was an error while login"),
        ));
    }

    let user_id: i32 = sqlx::query_scalar(r#"SELECT user_id FROM users WHERE user_nickname=$1"#)
    .bind(body.nickname.clone())
    .fetch_one(&pool.db)
    .await
    .expect("Not Found");

    if user_id < 1 {
        data.insert("nickname".to_string(), format!("Please re-check"));
        data.insert("password".to_string(), format!("Please re-check"));
        return HttpResponse::BadRequest().json(KeyValResponse::<String, String>::new(
            data,
            format!("Username and Password combination doesn't match !"),
        ));
    }

    let token: String = "random-sha256-hash-text".to_string(); // cryptography yet implemented
    let mut ip_addr: String = "0.0.0.0".to_string();
    let ua_addr: String = "".to_string();
    match req.peer_addr() {
        Some(addr) => {
            ip_addr = addr.ip().to_string();
        }
        None => {}
    }

    let query =
            sqlx::query(r#"INSERT INTO sessions (session_user_id, session_token, session_ip, session_ua, create_at) VALUES (?, ?, ?, ?, ?)"#)
                .bind(user_id)
                .bind(token.clone())
                .bind(ip_addr)
                .bind(ua_addr)
                .bind(Utc::now())
                .execute(&pool.db)
                .await
                .map_err(|e: sqlx::Error| e.to_string());

    if let Err(_) = query {
        //if e.contains("Duplicate entry") {
        //    return HttpResponse::BadRequest();
        //}

        return HttpResponse::InternalServerError().json(GenericResponse::<i8>::new(
            Vec::new(),
            "Server fault login error, please try again!".to_string(),
        ));
    }

    if query.is_ok() {
        return HttpResponse::Ok().json(GenericResponse::<LoginData>::new(
            vec!(LoginData { user_id: user_id.to_string(), token: token}),
            "Login success !!!".to_string(),
        ));
    }

    return HttpResponse::InternalServerError().json(NoDataResponse::new(
        "Server fault unknown error, please try again!".to_string(),
    ));
    
}
