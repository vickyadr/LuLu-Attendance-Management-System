use crate::models::m_login::UserData;
use actix_web::web::ReqData;
use sqlx::PgPool;

pub async fn is_login(pool: PgPool, bearer: Option<ReqData<String>>) -> Option<i32> {
    match bearer {
        Some(bearer) => {
            match sqlx::query_as::<_, UserData>(r#"SELECT users.user_fname, users.user_lname, users.user_level FROM users INNER JOIN sessions ON sessions.session_user_id = users.user_id WHERE sessions.session_token = $1"#)
                .bind(&bearer.into_inner())
                .fetch_one(&pool)
                .await
                {
                    Ok(data) => return Some(data.user_level),
                    Err(_) => return Some(0)
                }
        },
        None => None
    }
}