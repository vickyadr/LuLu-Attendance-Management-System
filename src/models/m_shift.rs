use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Shifts {
    #[serde(rename = "id")]
    pub shift_id: Option<i32>,
    #[serde(rename = "name")]
    pub shift_name: Option<String>,
    #[serde(rename = "start_time")]
    pub shift_start_time: Option<i64>,
    #[serde(rename = "end_time")]
    pub shift_end_time: Option<i64>,
    #[serde(rename = "start_enroll")]
    pub shift_start_enroll: Option<i64>,
    #[serde(rename = "end_enroll")]
    pub shift_end_enroll: Option<i64>,
    #[serde(rename = "passday")]
    pub shift_passday: Option<i16>,
}

