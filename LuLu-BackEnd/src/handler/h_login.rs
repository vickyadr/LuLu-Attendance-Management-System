use crate::{
    models::m_login::*, receiver::r_login::ReceiverLogin, utility::{
        authorization::is_login, hash::{create_key, encode_256}, stor::{AppState, GenericResponse, KeyValResponse, NoDataResponse}
    }
};
use actix_web::{get, post, web::{self, ReqData}, HttpResponse, Responder, HttpRequest};
use std::collections::HashMap;
use chrono::{Utc};

#[get("/login/user")]
pub async fn login_check(pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
    match bearer {
        Some(bearer) => {
                    match sqlx::query_as::<_, UserData>(r#"SELECT users.user_fname, users.user_lname, users.user_level FROM users INNER JOIN sessions ON sessions.session_user_id = users.user_id WHERE sessions.session_token = $1"#)
                    .bind(bearer.into_inner())
                    .fetch_one(&pool.db)
                    .await
                    {

                        Ok(data) => {
                            return HttpResponse::Ok().json(GenericResponse::<UserData>::single(
                                data,
                                format!("OK")
                            ));
                        },
                        Err(_e) => ()
                    }
        },
        _ => ()
    }

    HttpResponse::Ok().json(NoDataResponse::new(
        format!("Session token invalid"),
        400
    ))
}

#[get("/login/logout")]
pub async fn logout_act(pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {

    match is_login(pool.db.clone(), bearer.clone()).await
    {
        Some(level) =>{
            if level.lt(&100) {
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Sorry, doesnt have permission"),
                    403
                ))
            }
        },
        None => return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Session token invalid"),
                    401
                ))
    }

    match sqlx::query(r#"DELETE FROM sessions WHERE session_token = $1"#)
    .bind(bearer.unwrap().into_inner())
    .execute(&pool.db)
    .await
    {
        Ok(_) => return HttpResponse::Ok().json(NoDataResponse::new(
                format!("OK"),
                200
            )),
        Err(_) => return HttpResponse::Ok().json(NoDataResponse::new(
                format!("invalid session"),
                400
            ))
    }

}

#[post("/login")]
pub async fn login_act(pool: web::Data<AppState>, body: web::Json<ReceiverLogin>, req: HttpRequest) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    let ip_addr = match req.peer_addr(){
        Some(ip) => {
            ip.ip().to_string()
        },
        None => "".to_string()
    };

    let user_agent = match req.headers().get("user-agent"){
        Some(ua) => {
            ua.to_str().unwrap_or_default().to_string()
        },
        None => "".to_string() 
    };

    match body.nickname.clone() {
        Some(user) => {
            if user.len().lt(&3) {
                data.insert("nickname", format!("Username too sort, are you sure?"));
            }
        },
        None => {
            data.insert("nickname", format!("Please enter your username"));
        }
    }

    match body.password.clone() {
        Some(pass) => {
            if pass.len().lt(&8) {
                data.insert("password", format!("Password too sort, please think again"));
            }
        },
        None => {
            data.insert("password", format!("Please enter your password"));
        }
    }

    if data.len() > 0 {
        return HttpResponse::Ok().json(KeyValResponse::<&str, String>::new(
            data,
            format!("There was an error while login"),
            403
        ));
    }
    
    let password_encode = encode_256(body.password.clone().unwrap_or("".to_string()).leak()).await;
    let key = create_key(format!("SK-{}:{}", password_encode, Utc::now()).leak()).await;
    let token = key.private_key;

    //match sqlx::query_scalar::<_, i32>(r#"SELECT user_id FROM users WHERE user_nickname=$1 AND user_password=$2"#)
    match sqlx::query_scalar::<_, i32>(r#"INSERT INTO sessions (session_user_id, session_token, session_ip, session_ua) SELECT user_id, $1, $2, $3 FROM users WHERE user_nickname = $4 AND user_password = $5 RETURNING session_user_id"#)
    .bind(key.public_key)
    .bind(ip_addr)
    .bind(user_agent)
    .bind(body.nickname.clone())
    .bind(password_encode)
    .fetch_one(&pool.db)
    .await
    {
        Ok(user_id)=> return HttpResponse::Ok().json(GenericResponse::<LoginData>::ok(
                vec!(LoginData { user_id: user_id.to_string(), token: token}),
                format!("Login success !!!"),
            )),
        Err(_e)=>{
            data.insert("nickname", format!("Please re-check"));
            data.insert("password", format!("Please re-check"));
            return HttpResponse::Ok().json(KeyValResponse::<&str, String>::new(
                data,
                format!("Username and Password combination doesn't match !"),
                403
            ));
        }
    }
}
