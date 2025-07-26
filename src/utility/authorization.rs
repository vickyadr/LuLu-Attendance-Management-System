use crate::models::m_login::UserData;
use sqlx::PgPool;

pub async fn is_login(pool: PgPool, bearer: String) -> Option<i32> {
    
    match sqlx::query_as::<_, UserData>(r#"SELECT users.user_fname, users.user_lname, users.user_level FROM users INNER JOIN sessions ON sessions.session_user_id = users.user_id WHERE sessions.session_token = $1"#)
    .bind(&bearer)
    .fetch_one(&pool)
    .await
    {
        Ok(data) => {
            return Some(data.user_level);
        },
        Err(_) => ()
    }

    None
}