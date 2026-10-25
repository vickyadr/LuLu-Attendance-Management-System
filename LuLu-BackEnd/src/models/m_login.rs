use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct LoginData {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct UserData {
    // The nickname is the identity the permissions table is keyed on, and the UI
    // needs it to tell "editing myself" from "editing another account" when it
    // guards the self-lockout case.
    pub user_nickname: String,
    pub user_fname: String,
    pub user_lname: String,
}
