use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct LoginData {
    pub user_id: String,
    pub token: String,
}