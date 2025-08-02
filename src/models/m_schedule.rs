use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Schedules {
    #[serde(rename = "id")]
    pub schedule_id: Option<i32>,
    #[serde(rename = "name")]
    pub schedule_name: Option<String>,
    //#[serde(rename = "shift_id")]
    pub schedule_shift_id: Option<i32>,
    //#[serde(rename = "shift_name")]
    pub shift_name: Option<String>,
    #[serde(rename = "start")]
    pub shift_start_time: Option<i64>,
    #[serde(rename = "end")]
    pub shift_end_time: Option<i64>,
    #[serde(rename = "type")]
    pub schedule_type: Option<i32>,
    #[serde(rename = "dom")]
    pub schedule_dom: Option<i32>,
    #[serde(rename = "parrent")]
    pub schedule_parrent: Option<i32>,
}