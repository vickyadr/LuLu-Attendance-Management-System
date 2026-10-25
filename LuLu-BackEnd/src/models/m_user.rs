use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    pub user_id: i32,
    pub user_nickname: String,
    pub user_fname: String,
    pub user_lname: String,
    pub user_password: String,
}

/// One page a user can act on. The four action flags are stored separately and
/// all default to deny, so a page the user may read is never implicitly a page
/// they may write to.
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct UserPermission {
    pub page_key: String,
    pub can_view: bool,
    pub can_add: bool,
    pub can_edit: bool,
    pub can_delete: bool,
}

#[derive(Debug, Serialize)]
pub struct UserWithPermissions {
    pub user_id: i32,
    pub user_nickname: String,
    pub user_fname: String,
    pub user_lname: String,
    pub permissions: HashMap<String, UserPermission>,
}
