use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ReceiverLogin {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<bool>,
    /// Set by the employee PWA. Admin sessions never carry it, so an employee
    /// account cannot sign in through the admin form (and vice versa).
    #[serde(default)]
    pub field_app: Option<bool>,
}
