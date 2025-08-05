use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Devices {
    #[serde(rename = "id")]
    pub device_id: i32,
    #[serde(rename = "name")]
    pub device_name: String,
    #[serde(rename = "sn")]
    pub device_sn: String,
    #[serde(rename = "h_id")]
    pub device_handler_id: i32,
    #[serde(rename = "handler")]
    pub handler_name: String,
    #[serde(rename = "location")]
    pub device_location: String,
    #[serde(rename = "timezone")]
    pub device_timezone: i32,
    #[serde(rename = "status")]
    pub device_state: i32,
    #[serde(rename = "last_sync")]
    pub device_online: i64,
}
