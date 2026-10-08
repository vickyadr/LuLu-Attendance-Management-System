use crate::models::m_user::UserPermission;
use crate::utility::stor::IntoResponse;
use crate::utility::stor::NoDataResponse;
use actix_web::HttpResponse;
use actix_web::web::ReqData;
use sqlx::PgPool;
use std::collections::HashMap;

pub const PAGE_KEYS: [&str; 7] = [
    "dashboard",
    "employee",
    "shift",
    "report",
    "controller",
    "settings",
    "users",
];


pub const SESSION_IDLE_SECS: i64 = 60 * 60 * 12;

macro_rules! live_session_predicate {
    () => {
        "sessions.session_token = $1 \
         AND sessions.update_at > EXTRACT(EPOCH FROM NOW())::BIGINT - $2"
    };
}

pub const LIVE_SESSION: &str = live_session_predicate!();

pub const SESSION_USER_LOOKUP: &str = concat!(
    "SELECT users.user_nickname FROM users \
     INNER JOIN sessions ON sessions.session_user_id = users.user_id \
     WHERE ",
    live_session_predicate!()
);

pub const SESSION_USER_DETAIL_LOOKUP: &str = concat!(
    "SELECT users.user_nickname, users.user_fname, users.user_lname \
     FROM users INNER JOIN sessions ON sessions.session_user_id = users.user_id \
     WHERE ",
    live_session_predicate!()
);

pub const SESSION_REFRESH_AFTER: i64 = SESSION_IDLE_SECS / 4;

pub fn client_ip(addr: Option<std::net::SocketAddr>) -> String {
    let Some(addr) = addr else {
        return String::new();
    };
    match addr.ip() {
        std::net::IpAddr::V4(v4) => v4.to_string(),
        std::net::IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => v4.to_string(),
            None => v6.to_string(),
        },
    }
}


pub async fn is_login_token(pool: PgPool, token: String) -> Option<String> {

    let nickname: Option<String> = sqlx::query_scalar::<_, String>(SESSION_USER_LOOKUP)
        .bind(&token)
        .bind(SESSION_IDLE_SECS)
        .fetch_one(&pool)
        .await
        .ok();

    if nickname.is_some()
        && let Err(err) = sqlx::query(
            r#"UPDATE sessions SET update_at = EXTRACT(EPOCH FROM NOW())::BIGINT
               WHERE session_token = $1
                 AND update_at < EXTRACT(EPOCH FROM NOW())::BIGINT - $2"#,
        )
        .bind(&token)
        .bind(SESSION_REFRESH_AFTER)
        .execute(&pool)
        .await
    {

        log::warn!("session refresh failed: {err}");
    }

    nickname
}

pub async fn is_login(pool: PgPool, bearer: Option<ReqData<String>>) -> Option<String> {
    match bearer {
        Some(bearer) => is_login_token(pool, bearer.into_inner()).await,
        None => None,
    }
}

pub async fn get_user_permissions(
    pool: PgPool,
    user_nickname: &str,
) -> HashMap<String, UserPermission> {
    match sqlx::query_as::<_, UserPermission>(
        r#"SELECT page_key, can_view, can_add, can_edit, can_delete
           FROM user_page_permissions WHERE user_nickname = $1"#,
    )
    .bind(user_nickname)
    .fetch_all(&pool)
    .await
    {
        Ok(perms) => perms.into_iter().map(|p| (p.page_key.clone(), p)).collect(),
        Err(_) => HashMap::new(),
    }
}

/// The user nickname behind a session token, or None when the session is dead.
pub async fn get_session_user(pool: PgPool, bearer: Option<ReqData<String>>) -> Option<String> {
    match bearer {
        Some(bearer) => is_login_token(pool, bearer.into_inner()).await,
        None => None,
    }
}

/// The four permissions a page can carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    View,
    Add,
    Edit,
    Delete,
}

impl Action {
    fn message(self) -> &'static str {
        match self {
            Action::View => "Sorry, doesnt have permission",
            Action::Add => "Sorry, doesnt have add permission",
            Action::Edit => "Sorry, doesnt have edit permission",
            Action::Delete => "Sorry, doesnt have delete permission",
        }
    }
}

/// Gate for one specific action on a page.
#[allow(clippy::result_unit_err)]
pub async fn check_action_access(
    pool: PgPool,
    bearer: Option<ReqData<String>>,
    page_key: &str,
    action: Action,
) -> Result<HttpResponse, ()> {
    let user_nickname = match get_session_user(pool.clone(), bearer).await {
        Some(nickname) => nickname,
        None => {
            return Ok(
                NoDataResponse::new("Session token invalid".to_string(), 401).into_response(),
            );
        }
    };

    let flags: Option<(bool, bool, bool, bool)> = sqlx::query_as(
        r#"SELECT can_view, can_add, can_edit, can_delete
           FROM user_page_permissions WHERE user_nickname = $1 AND page_key = $2"#,
    )
    .bind(&user_nickname)
    .bind(page_key)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let allowed = match flags {
        Some((can_view, can_add, can_edit, can_delete)) => match action {
            Action::View => can_view,
            Action::Add => can_add,
            Action::Edit => can_edit,
            Action::Delete => can_delete,
        },
        None => false,
    };

    if allowed {
        Err(())
    } else {
        Ok(NoDataResponse::new(action.message().to_string(), 403).into_response())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sock(ip: &str) -> Option<std::net::SocketAddr> {

        let literal = if ip.contains(':') {
            format!("[{ip}]:4444")
        } else {
            format!("{ip}:4444")
        };
        literal.parse().ok()
    }

    #[test]
    fn a_plain_ipv4_client_keeps_its_dotted_quad() {
        assert_eq!(client_ip(sock("10.55.54.51")), "10.55.54.51");
        assert_eq!(client_ip(sock("127.0.0.1")), "127.0.0.1");
    }

    #[test]
    fn an_ipv4_client_on_a_dual_stack_socket_is_unwrapped_not_left_mapped() {

        let mapped: std::net::IpAddr = "::ffff:10.55.54.51".parse().unwrap();
        assert!(mapped.is_ipv6());

        let addr = std::net::SocketAddr::new(mapped, 4444);
        let stored = client_ip(Some(addr));

        assert_eq!(stored, "10.55.54.51");
        assert!(
            stored.len() <= 45,
            "IPv4 traffic must not reach the length limit that broke it"
        );
    }

    #[test]
    fn a_real_ipv6_client_is_stored_in_full() {
        assert_eq!(client_ip(sock("2001:db8::1")), "2001:db8::1");
        assert_eq!(client_ip(sock("fe80::1")), "fe80::1");
    }

    #[test]
    fn every_stored_address_fits_the_widened_column() {

        let cases = [
            "10.55.54.51",
            "255.255.255.255",
            "::1",
            "::ffff:255.255.255.255",
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
        ];
        for case in cases {
            let stored = client_ip(sock(case));
            assert!(
                stored.chars().count() <= 45,
                "{case} stored as {stored:?} exceeds the column limit"
            );
        }
    }

    #[test]
    fn a_missing_peer_address_stores_an_empty_string() {

        assert_eq!(client_ip(None), "");
    }

    #[test]
    fn the_live_session_predicate_binds_both_the_token_and_the_window() {

        assert!(LIVE_SESSION.contains("$1"), "token must be the first bind");
        assert!(
            LIVE_SESSION.contains("$2"),
            "the idle window must be the second bind"
        );
        assert!(
            LIVE_SESSION.contains("update_at"),
            "an expiry check that never reads update_at is not an expiry check"
        );
    }

    #[test]
    fn every_authentication_query_carries_the_expiry() {

        for (name, query) in [
            ("SESSION_USER_LOOKUP", SESSION_USER_LOOKUP),
            ("SESSION_USER_DETAIL_LOOKUP", SESSION_USER_DETAIL_LOOKUP),
        ] {
            assert!(
                query.contains("update_at > EXTRACT(EPOCH FROM NOW())"),
                "{name} does not apply the idle window"
            );
            assert!(
                query.contains("- $2"),
                "{name} never subtracts the bound window"
            );

            assert_eq!(
                query.matches("$1").count(),
                1,
                "{name} should reference the token exactly once"
            );
        }
    }

    #[test]
    fn the_refresh_threshold_is_derived_from_the_idle_window() {
        assert_eq!(SESSION_REFRESH_AFTER * 4, SESSION_IDLE_SECS);
    }
}
