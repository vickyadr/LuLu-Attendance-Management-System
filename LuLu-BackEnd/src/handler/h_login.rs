use crate::{
    models::m_login::*,
    receiver::r_login::ReceiverLogin,
    utility::{
        hash::{hash_password, needs_rehash, verify_password},
        stor::{AppState, GenericResponse, IntoResponse, KeyValResponse, NoDataResponse},
    },
};
use actix_web::{
    HttpRequest, Responder, get, post,
    web::{self, ReqData},
};
use std::collections::HashMap;

#[get("/login/user")]
pub async fn login_check(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Some(bearer) = bearer {

        match sqlx::query_as::<_, UserData>(
            crate::utility::authorization::SESSION_USER_DETAIL_LOOKUP,
        )
        .bind(bearer.into_inner())
        .bind(crate::utility::authorization::SESSION_IDLE_SECS)
        .fetch_one(&pool.db)
        .await
        {
            Ok(data) => {
                return GenericResponse::<UserData>::single(data, "OK".to_string()).into_response();
            }
            Err(_e) => (),
        }
    }

    NoDataResponse::new("Session token invalid".to_string(), 400).into_response()
}

#[get("/login/logout")]
pub async fn logout_act(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {

    let token = match bearer {
        Some(bearer) => bearer.into_inner(),
        None => {
            return NoDataResponse::new("Session token invalid".to_string(), 401).into_response();
        }
    };

    match sqlx::query(r#"DELETE FROM sessions WHERE session_token = $1"#)
        .bind(token)
        .execute(&pool.db)
        .await
    {
        Ok(_) => NoDataResponse::new("OK".to_string(), 200).into_response(),
        Err(err) => {
            log::error!("logout failed to delete session: {err}");
            NoDataResponse::new("invalid session".to_string(), 400).into_response()
        }
    }
}

#[post("/login")]
pub async fn login_act(
    pool: web::Data<AppState>,
    body: web::Json<ReceiverLogin>,
    req: HttpRequest,
) -> impl Responder {
    let mut data: HashMap<&str, String> = HashMap::new();

    let ip_addr = crate::utility::authorization::client_ip(req.peer_addr());

    let user_agent = match req.headers().get("user-agent") {
        Some(ua) => ua.to_str().unwrap_or_default().to_string(),
        None => "".to_string(),
    };

    const MAX_UA_CHARS: usize = 1024;
    let user_agent = if user_agent.len() > MAX_UA_CHARS {
        log::warn!(
            "user-agent header is {} bytes, storing the first {MAX_UA_CHARS}",
            user_agent.len()
        );
        user_agent.chars().take(MAX_UA_CHARS).collect()
    } else {
        user_agent
    };

    match body.nickname.clone() {
        Some(user) => {
            if user.len().lt(&3) {
                data.insert("nickname", "Username too sort, are you sure?".to_string());
            }
        }
        None => {
            data.insert("nickname", "Please enter your username".to_string());
        }
    }

    match body.password.clone() {
        Some(pass) => {
            if pass.len().lt(&8) {
                data.insert(
                    "password",
                    "Password too sort, please think again".to_string(),
                );
            }
        }
        None => {
            data.insert("password", "Please enter your password".to_string());
        }
    }

    if !data.is_empty() {
        return KeyValResponse::<&str, String>::new(
            data,
            "There was an error while login".to_string(),
            403,
        )
        .into_response();
    }

    let clear = body.password.clone().unwrap_or_default();

    let stored: Option<(i32, String)> =
        sqlx::query_as(r#"SELECT user_id, user_password FROM users WHERE user_nickname = $1"#)
            .bind(body.nickname.clone())
            .fetch_optional(&pool.db)
            .await
            .ok()
            .flatten();

    let (user_id, stored_hash) = match stored {
        Some(row) => row,
        None => return login_failed(data),
    };

    if !verify_password(&stored_hash, &clear).await {
        return login_failed(data);
    }

    if needs_rehash(&stored_hash) {
        let upgraded = hash_password(&clear).await;
        let _ = sqlx::query(r#"UPDATE users SET user_password = $1 WHERE user_id = $2"#)
            .bind(upgraded)
            .bind(user_id)
            .execute(&pool.db)
            .await;
    }

    let mut buf = [0u8; 32];
    if getrandom::getrandom(&mut buf).is_err() {
        return NoDataResponse::new("Internal server error".to_string(), 500).into_response();
    }
    let token = hex::encode(buf);
    let stored_token = crate::utility::hash::encode_256(&token).await;

    match sqlx::query_scalar::<_, i32>(
        r#"INSERT INTO sessions (session_user_id, session_token, session_ip, session_ua) VALUES ($1, $2, $3, $4) RETURNING session_user_id"#,
    )
    .bind(user_id)
    .bind(stored_token)
    .bind(ip_addr)
    .bind(user_agent)
    .fetch_one(&pool.db)
    .await
    {
        Ok(user_id) => GenericResponse::<LoginData>::ok(
            vec![LoginData {
                user_id: user_id.to_string(),
                token,
            }],
            "Login success !!!".to_string(),
        )
        .into_response(),
        Err(e) => {
            log::error!("Error: {}", e);
            NoDataResponse::new("Internal server error".to_string(), 500).into_response()
        }
    }
}

fn login_failed(mut data: HashMap<&str, String>) -> actix_web::HttpResponse {
    data.insert("nickname", "Please re-check".to_string());
    data.insert("password", "Please re-check".to_string());
    KeyValResponse::<&str, String>::new(
        data,
        "Username and Password combination doesn't match !".to_string(),
        403,
    )
    .into_response()
}
