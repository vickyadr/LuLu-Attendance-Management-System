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

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct ScheduleHelper {
    pub schedule_name: String,
    pub shift_name: String,
    pub shift_id: i32,
    pub schedule_dom: i32,
    pub schedule_parrent: i32,
    pub schedule_type: i32,
    pub schedule_hols: i16,
    pub shift_start_time: i64,
    pub shift_end_time: i64,
    pub shift_start_enroll: i64,
    pub shift_end_enroll: i64,
    pub shift_passday: i16,
    pub shift_prevday: i16,
}