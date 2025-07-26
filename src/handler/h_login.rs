use crate::{
    receiver::r_login::ReceiverLogin,
    utility::{
        stor::{AppState, GenericResponse, KeyValResponse, NoDataResponse},
        hash::{create_key, encode_256},
    },
    models::{m_login::*},
};
use actix_web::{get, post, web::{self, ReqData}, HttpResponse, Responder, HttpRequest};
//use actix_web_httpauth::{Auth, Bearer};
use std::collections::HashMap;
use chrono::{Utc};
use sqlx::Postgres;

#[get("/login/user")]
pub async fn login_check(pool: web::Data<AppState>, bearer: Option<ReqData<String>>) -> impl Responder {
    //let bear = encode_256("3fc95b39b3757a834efa8dbe01bd7f7db86747872885f6d936e37f1de74829af").await;
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

    match bearer {
        Some(bearer) => {
            match sqlx::query(r#"DELETE FROM sessions WHERE session_token = $1"#)
                .bind(bearer.into_inner())
                .execute(&pool.db)
                .await
                {
                    Ok(_affected) => {
                        return HttpResponse::Ok().json(NoDataResponse::new(
                            format!("OK"),
                            200
                        ));
                    },
                    Err(_e) => ()
                }
        },
        None =>()
    }

    HttpResponse::Ok().json(NoDataResponse::new(
        format!("Session token invalid"),
        400
    ))
}

#[post("/login")]
pub async fn login_act(pool: web::Data<AppState>, body: web::Json<ReceiverLogin>, req: HttpRequest) -> impl Responder {
    let mut data: HashMap<String, String> = HashMap::new();

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
    
    //println!("{}", encode_256("demo12345").await);

    match body.nickname.clone() {
        Some(user) => {
            if user.len().lt(&3) {
                data.insert("nickname".to_string(), format!("Username too sort, are you sure?"));
            }
        },
        None => {
            data.insert("nickname".to_string(), format!("Please enter your username"));
        }
    }

    match body.password.clone() {
        Some(pass) => {
            if pass.len().lt(&8) {
                data.insert("password".to_string(), format!("Password too sort, please think again"));
            }
        },
        None => {
            data.insert("password".to_string(), format!("Please enter your password"));
        }
    }

    if data.len() > 0 {
        return HttpResponse::Ok().json(KeyValResponse::<String, String>::new(
            data,
            format!("There was an error while login"),
            403
        ));
    }
    
    let password_encode = encode_256(body.password.clone().unwrap_or("".to_string()).leak()).await;

    match sqlx::query_scalar::<Postgres, i32>(r#"SELECT user_id FROM users WHERE user_nickname=$1 AND user_password=$2"#)
    .bind(body.nickname.clone())
    .bind(password_encode.clone())
    .fetch_one(&pool.db)
    .await
    {
        Ok(user_id) =>{            
            let key = create_key(format!("SK-{}:{}", password_encode, Utc::now()).leak()).await;

            println!("PRIVATE : {}", key.private_key);
            println!("PUBLIC : {}", key.public_key);

            let token: String = key.private_key;
            //let mut ip_addr: String = "0.0.0.0".to_string();
            //let ua_addr: String = "UA".to_string();
            /*match req.peer_addr() {
                Some(addr) => {
                    ip_addr = addr.ip().to_string();
                }
                None => {}
            }*/

            let query =
                    sqlx::query(r#"INSERT INTO sessions (session_user_id, session_token, session_ip, session_ua) VALUES ($1, $2, $3, $4)"#)
                        .bind(user_id)
                        .bind(key.public_key)
                        .bind(ip_addr)
                        .bind(user_agent)
                        //.bind(Utc::now())
                        .execute(&pool.db)
                        .await
                        .map_err(|e: sqlx::Error| e.to_string());

            if let Err(_e) = query {
                //if e.contains("Duplicate entry") {
                //    return HttpResponse::BadRequest();
                //}
                //println!("{}",e);
                return HttpResponse::Ok().json(NoDataResponse::new(
                    format!("Server fault login error, please try again!"),
                    500
                ));
            }

            if query.is_ok() {
                return HttpResponse::Ok().json(GenericResponse::<LoginData>::ok(
                    vec!(LoginData { user_id: user_id.to_string(), token: token}),
                    format!("Login success !!!"),
                ));
            }

            return HttpResponse::Ok().json(NoDataResponse::new(
                format!("Server fault unknown error, please try again!"),
                500
            ));
            
        },
        Err(_e)=>{
            data.insert("nickname".to_string(), format!("Please re-check"));
            data.insert("password".to_string(), format!("Please re-check"));
            return HttpResponse::Ok().json(KeyValResponse::<String, String>::new(
                data,
                format!("Username and Password combination doesn't match !"),
                403
            ));
        }
    }
}
