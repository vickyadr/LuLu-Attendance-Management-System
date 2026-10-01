use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Settings {
    pub setting_key: String,
    pub setting_value: i64,
}

#[derive(Deserialize, Debug)]
pub struct ReceiverSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub late_tolerance_sec: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overtime_tolerance_sec: Option<i64>,
}
