use crate::models::m_user::UserPermission;
use crate::utility::stor::IntoResponse;
use crate::utility::stor::NoDataResponse;
use actix_web::HttpResponse;
use actix_web::web::ReqData;
use sqlx::PgPool;
use std::collections::HashMap;

/// Pages the frontend can gate on. Kept in one place so a new page cannot be
/// added to the UI without the backend recognising its permission key.
pub const PAGE_KEYS: [&str; 9] = [
    "dashboard",
    "employee",
    "shift",
    "report",
    "controller",
    "settings",
    "users",
    "leave",
    "payroll",
];

/// How long a session stays valid after it was last used, in seconds.
///
/// Nothing expired sessions before: the lookup below matched any row in
/// `sessions`, so a token kept working forever. Anyone who captured one - a
/// shared machine, a proxy log, a leaked screenshot of the devtools panel -
/// had permanent access, and deleting the row was the only revocation.
///
/// The window is sliding: an authenticated request moves `update_at` forward,
/// so an operator who leaves the dashboard open all day is not logged out
/// mid-shift, while an abandoned tab stops working on its own.
pub const SESSION_IDLE_SECS: i64 = 60 * 60 * 12;

/// The `sessions` predicate every authentication query must carry.
///
/// Both the middleware path and `/login/user` read this table, so the expiry
/// lives in one place: a new caller that rolls its own `WHERE` silently
/// resurrects eternal tokens.
///
/// A macro rather than a plain `const` because the full queries below are built
/// with `concat!`, which accepts literals only. Sharing the fragment this way
/// keeps every authentication query on one predicate while still producing a
/// `&'static str` - the only type sqlx accepts without an explicit
/// `AssertSqlSafe`, whose compile error is a deliberate speed bump against
/// smuggling request input into a query string.
macro_rules! live_session_predicate {
    () => {
        "sessions.session_token = $1 \
         AND sessions.update_at > EXTRACT(EPOCH FROM NOW())::BIGINT - $2"
    };
}

/// The predicate on its own, for tests and any caller that composes its own SQL.
pub const LIVE_SESSION: &str = live_session_predicate!();

/// Session lookup used by the middleware and the WebSocket upgrade.
pub const SESSION_USER_LOOKUP: &str = concat!(
    "SELECT users.user_nickname FROM users \
     INNER JOIN sessions ON sessions.session_user_id = users.user_id \
     WHERE ",
    live_session_predicate!()
);

/// The same lookup widened to the fields `/login/user` returns.
///
/// It lives here so the expiry cannot be applied to one query and forgotten on
/// the other: that route reads `sessions` with its own SQL, so a token every
/// other endpoint rejects would still answer 200 there and the frontend would
/// render a logged-in shell with no data behind it.
pub const SESSION_USER_DETAIL_LOOKUP: &str = concat!(
    "SELECT users.user_nickname, users.user_fname, users.user_lname \
     FROM users INNER JOIN sessions ON sessions.session_user_id = users.user_id \
     WHERE users.user_field_only = FALSE AND ",
    live_session_predicate!()
);

/// Session lookup for admin-only transports (the WebSocket feed). The regular
/// `SESSION_USER_LOOKUP` must keep resolving field users so the HTTP gate can
/// answer them with its own 403.
pub const SESSION_ADMIN_LOOKUP: &str = concat!(
    "SELECT users.user_nickname FROM users \
     INNER JOIN sessions ON sessions.session_user_id = users.user_id \
     WHERE users.user_field_only = FALSE AND ",
    live_session_predicate!()
);

/// Keep a live session's `update_at` moving so active use extends its life.
///
/// Guarded by `update_at < now - SESSION_REFRESH_AFTER`, which turns the write
/// into roughly one row update per session per few hours instead of one per
/// request; the pool would otherwise take a write on every page load.
///
/// Derived from the idle window rather than written as its own number on
/// purpose. It must stay strictly below it: at or past the idle timeout a
/// session refreshes itself into eternal life, because every authenticated
/// request pushes `update_at` forward before the window can elapse. Expressed as
/// a division, that invariant holds for any future change to
/// `SESSION_IDLE_SECS` instead of needing a test to catch it.
pub const SESSION_REFRESH_AFTER: i64 = SESSION_IDLE_SECS / 4;

/// The client address as written into `sessions.session_ip`.
///
/// A server bound to an IPv6 socket (dual-stack, the default) reports an IPv4
/// client as `::ffff:10.55.54.51`. That is 18 characters and did not fit
/// `VARCHAR(16)`, so every login from a LAN client died with
/// "value too long for type character varying(16)" - a 500 with no session
/// row, i.e. nobody could log in at all.
///
/// Unwrapping the mapped form back to a dotted quad keeps the column readable
/// and under the limit for IPv4 traffic, while the widened column still has to
/// hold a genuine IPv6 address. Returns an empty string when the address is
/// unavailable, which the NOT NULL column accepts.
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

/// Confirm a token maps to a live session. Returns the user nickname.
/// The WebSocket upgrade arrives as `?token=` because the browser cannot attach
/// an `Authorization` header there.
pub async fn is_login_token(pool: PgPool, token: String) -> Option<String> {
    session_token_user(pool, token, SESSION_USER_LOOKUP).await
}

/// Same checks as `is_login_token`, but a field-only account answers `None`.
pub async fn is_admin_token(pool: PgPool, token: String) -> Option<String> {
    session_token_user(pool, token, SESSION_ADMIN_LOOKUP).await
}

async fn session_token_user(pool: PgPool, token: String, lookup: &'static str) -> Option<String> {
    // A `&'static str` query, so sqlx needs no `AssertSqlSafe`: this is the
    // compile-time-checked path, and a future edit that introduces request data
    // here fails to build instead of shipping.
    let nickname: Option<String> = sqlx::query_scalar::<_, String>(lookup)
        .bind(&token)
        .bind(SESSION_IDLE_SECS)
        .fetch_one(&pool)
        .await
        .ok();

    // Only extend a session that actually verified. Refreshing first would keep
    // a rejected token alive indefinitely by restarting its window.
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
        // A failed refresh must not fail the request: the session is valid, it
        // will simply expire on schedule instead of sliding.
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

/// All page permissions for a user as a HashMap<page_key, UserPermission>.
/// Empty on error or when the user has no rows - every caller must then deny.
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
///
/// The row is read once and the enum picks the flag in Rust, so there is no
/// string-built SQL anywhere near user input: page_key is always a bound
/// parameter and only the match below touches the enum.
/// `Err(())` means "allowed" and `Ok(response)` carries the denial - this reads
/// backwards versus normal Result use, but it is the gate convention every handler
/// in this repo shares (`if let Ok(resp) = ... { return resp; }`).
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

    // A field token exists only for the employee app. Even if one is presented
    // here, it must not open an admin page.
    let field_only = sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(user_field_only, FALSE) FROM users WHERE user_nickname = $1",
    )
    .bind(&user_nickname)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten()
    .unwrap_or(false);
    if field_only && page_key != "field" {
        return Ok(NoDataResponse::new(
            "This account is for the employee app".to_string(),
            403,
        )
        .into_response());
    }

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

    // Missing row or a failed read means deny. The permission table is the only
    // source of access: nothing falls back to allowing.
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
        // An IPv6 address has to be bracketed before the port; `2001:db8::1:4444`
        // parses as neither an address nor a socket.
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
        // This is the shape that broke login: a server bound to "::" reports an
        // IPv4 client as ::ffff:10.55.54.51, 18 characters, which did not fit
        // the old VARCHAR(16) and turned every LAN login into a 500.
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
        // The column is VARCHAR(45): the longest address PostgreSQL's inet
        // renders is a full IPv6 with an embedded IPv4 suffix.
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
        // session_ip is NOT NULL; the column is what the INSERT binds, so this
        // must never become a null or a panic.
        assert_eq!(client_ip(None), "");
    }

    #[test]
    fn the_live_session_predicate_binds_both_the_token_and_the_window() {
        // Every query that authenticates a bearer interpolates this string, so a
        // missing parameter placeholder would not fail to compile -- it would
        // fail at runtime, for every request, as "bind message supplies 1
        // parameters but prepared statement requires 2".
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
        // Three routes read `sessions`: the middleware (via `is_login_token`, used
        // by `check_action_access`, `get_session_user` and the WebSocket upgrade),
        // and `/login/user`. They are separate queries, so an expiry applied to
        // only some of them leaves a bypass: a dead token gets 200 on the routes
        // that were not updated and the frontend renders a logged-in shell with
        // no data behind it.
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
            // A query with two placeholders bound one parameter fails at runtime
            // for every request, not at compile time.
            assert_eq!(
                query.matches("$1").count(),
                1,
                "{name} should reference the token exactly once"
            );
        }
    }

    #[test]
    fn the_refresh_threshold_is_derived_from_the_idle_window() {
        // `SESSION_REFRESH_AFTER` is written as `SESSION_IDLE_SECS / 4`, so
        // "refreshes before the window elapses" is structural rather than a
        // convention a future edit can quietly break.
        assert_eq!(SESSION_REFRESH_AFTER * 4, SESSION_IDLE_SECS);
    }
}
