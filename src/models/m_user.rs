use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    pub user_id: String,
    pub user_nickname: String,
    pub user_fname: String,
    pub user_lname: String,
    pub user_level: i32,
    pub user_password: String,
    pub update_at: String,
    pub create_at: String,
}