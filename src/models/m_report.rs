use serde::{Deserialize, Serialize};
//use chrono::{NaiveDate, NaiveTime};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Reports {
    #[serde(rename = "id")]
    pub employee_id: Option<i32>,
    #[serde(rename = "fist_name")]
    pub employee_fname: Option<String>,
    #[serde(rename = "last_name")]
    pub employee_lname: Option<String>,
    #[serde(rename = "departement")]
    pub employee_departement: Option<String>,
    pub enroll_dow: Option<f64>,
    #[serde(rename = "date")]
    pub enroll_date: Option<f64>,
    #[serde(rename = "tz")]
    pub device_timezone: Option<i32>,
    #[serde(rename = "enroll_device")]
    pub device_name: Option<String>,
    #[serde(rename = "enroll_location")]
    pub device_location: Option<String>,
    #[serde(rename = "shift_start")]
    pub shift_start_time: Option<i64>,
    #[serde(rename = "shift_end")]
    pub shift_end_time: Option<i64>,
    //#[serde(rename = "shift_name")]
    pub shift_name: Option<String>,
    //#[serde(rename = "schedule_name")]
    pub schedule_name: Option<String>,
    #[serde(rename = "start")]
    pub start_enroll: Option<i64>,
    #[serde(rename = "end")]
    pub end_enroll: Option<i64>,
}