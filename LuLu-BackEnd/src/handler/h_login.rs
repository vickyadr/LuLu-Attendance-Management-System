use crate::{
    models::m_login::*,
    receiver::r_login::ReceiverLogin,
    utility::{
        hash::{hash_password, needs_rehash, verify_password},
        stor::{AppState, GenericResponse, IntoResponse, KeyValResponse, NoDataResponse},
    },
};
use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post,
    web::{self, ReqData},
};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const LOGIN_MAX_FAILS: u32 = 10;
const LOGIN_WINDOW_SECS: i64 = 15 * 60;

#[derive(Default)]
struct LoginFails {
    count: u32,
    first_at: i64,
}

fn login_fails() -> &'static Mutex<HashMap<String, LoginFails>> {
    static TABLE: OnceLock<Mutex<HashMap<String, LoginFails>>> = OnceLock::new();
    TABLE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Seconds the caller must wait, or 0 when it may try now. One process, one
/// map: enough for a single attendance server, and no schema for a counter.
fn login_throttle_wait(key: &str) -> i64 {
    let now = now_epoch();
    let mut table = match login_fails().lock() {
        Ok(table) => table,
        Err(poisoned) => poisoned.into_inner(),
    };
    let Some(entry) = table.get(key) else { return 0 };
    if now - entry.first_at >= LOGIN_WINDOW_SECS {
        table.remove(key);
        return 0;
    }
    if entry.count < LOGIN_MAX_FAILS {
        return 0;
    }
    LOGIN_WINDOW_SECS - (now - entry.first_at)
}

fn record_login_failure(key: &str) {
    let now = now_epoch();
    let mut table = match login_fails().lock() {
        Ok(table) => table,
        Err(poisoned) => poisoned.into_inner(),
    };
    let entry = table.entry(key.to_string()).or_default();
    if now - entry.first_at >= LOGIN_WINDOW_SECS {
        entry.count = 0;
        entry.first_at = now;
    }
    entry.count += 1;
}

fn clear_login_failures(key: &str) {
    if let Ok(mut table) = login_fails().lock() {
        table.remove(key);
    }
}

#[get("/login/user")]
pub async fn login_check(
    pool: web::Data<AppState>,
    bearer: Option<ReqData<String>>,
) -> impl Responder {
    if let Some(bearer) = bearer {
        // Shares `SESSION_USER_DETAIL_LOOKUP` with the middleware path on purpose, and
        // that route lives in `authorization` so the expiry cannot be applied to
        // one query and forgotten on the other: this route reads `sessions` with
        // its own SQL, so a token every other endpoint rejects would answer 200
        // here and the frontend would render a logged-in shell with no data.
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
    // No expiry check here on purpose: the point of logout is to destroy the
    // row, and that has to work for a session that has already lapsed.
    //
    // `bearer.unwrap()` used to sit above this and panicked the worker when the
    // header was absent. The route sits inside the authenticated scope, so the
    // middleware rejects that first - but a panic in a request handler kills
    // the worker rather than answering 401, and a 500 to an unauthenticated
    // caller is a worse lie than a 401.
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

    // `session_ua` is VARCHAR(1024) but the header is client-supplied and can
    // run to several kilobytes. An over-long value used to abort the INSERT with
    // "value too long for type character varying(...)" and answer 500, so a
    // login failed for a reason unrelated to the credentials. Truncate to the
    // column limit by characters, which never splits a UTF-8 boundary because
    // `to_str()` already returned valid UTF-8.
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
    let throttle_key = format!("{}|{}", body.nickname.clone().unwrap_or_default(), ip_addr);
    let wait = login_throttle_wait(&throttle_key);
    if wait > 0 {
        log::warn!(
            "login throttled for {} from {} ({}s left)",
            body.nickname.clone().unwrap_or_default(),
            ip_addr,
            wait
        );
        return HttpResponse::TooManyRequests()
            .insert_header(("Retry-After", wait.to_string()))
            .json(NoDataResponse::new(
                "Too many attempts, try again later".to_string(),
                429,
            ));
    }

    // Fetch the stored hash first so Argon2 verification happens in Rust,
    // not as a column comparison inside the INSERT...SELECT.
    let stored: Option<(i32, String)> =
        sqlx::query_as(r#"SELECT user_id, user_password FROM users WHERE user_nickname = $1"#)
            .bind(body.nickname.clone())
            .fetch_optional(&pool.db)
            .await
            .ok()
            .flatten();

    let (user_id, stored_hash) = match stored {
        Some(row) => row,
        None => {
            // Argon2 on the missing-user path too: otherwise the response time
            // is an account-existence oracle.
            let _ = hash_password("no-such-user").await;
            record_login_failure(&throttle_key);
            log::warn!("login failed for unknown nickname from {ip_addr}");
            return login_failed(data);
        }
    };

    if !verify_password(&stored_hash, &clear).await {
        record_login_failure(&throttle_key);
        log::warn!(
            "login failed for {} from {ip_addr}",
            body.nickname.clone().unwrap_or_default()
        );
        return login_failed(data);
    }

    // Two account kinds share this table but not this form: employee PWA
    // credentials must sign in through the field app, admin credentials must
    // not. A mismatch answers the same generic failure as a bad password.
    let field_only = sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(user_field_only, FALSE) FROM users WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&pool.db)
    .await
    .ok()
    .flatten()
    .unwrap_or(false);
    if field_only != body.field_app.unwrap_or(false) {
        record_login_failure(&throttle_key);
        log::warn!(
            "login refused by account kind for {} from {ip_addr}",
            body.nickname.clone().unwrap_or_default()
        );
        return login_failed(data);
    }

    clear_login_failures(&throttle_key);

    // Lazy re-hash: a legacy SHA-256 row that just verified gets upgraded
    // to Argon2 on this successful login, so migration needs no downtime.
    if needs_rehash(&stored_hash) {
        let upgraded = hash_password(&clear).await;
        let _ = sqlx::query(r#"UPDATE users SET user_password = $1 WHERE user_id = $2"#)
            .bind(upgraded)
            .bind(user_id)
            .execute(&pool.db)
            .await;
    }

    // Session token from OS entropy, hex-encoded for the VARCHAR column.
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
