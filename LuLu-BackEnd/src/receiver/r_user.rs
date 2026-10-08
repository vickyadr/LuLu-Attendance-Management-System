use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ReceiverUserAdd {
    pub nickname: Option<String>,
    pub password: Option<String>,
    pub fname: Option<String>,
    pub lname: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReceiverUserEdit {
    pub user_nickname: Option<String>,
    pub nickname: Option<String>,
    pub password: Option<String>,
    pub fname: Option<String>,
    pub lname: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReceiverUserDelete {
    pub user_nickname: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReceiverUserPermissionsUpdate {
    pub user_nickname: Option<String>,
    pub permissions: Vec<ReceiverPermission>,
}

#[derive(Debug, Deserialize)]
pub struct ReceiverPermission {
    pub page_key: String,
    pub can_view: bool,
    pub can_add: bool,
    pub can_edit: bool,
    pub can_delete: bool,
}
