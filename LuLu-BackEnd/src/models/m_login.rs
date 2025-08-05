use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct LoginData {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct UserData {
    pub user_fname: String,
    pub user_lname: String,
    pub user_level: i32,
}