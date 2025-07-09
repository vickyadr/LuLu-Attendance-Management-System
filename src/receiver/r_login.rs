use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ReceiverLogin {
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<bool>,
}
