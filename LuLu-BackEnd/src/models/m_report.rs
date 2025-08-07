use serde::{Deserialize, Serialize};
//use chrono::{NaiveDate, NaiveTime};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Reports {
    #[serde(rename = "id")]
    pub employee_id: i32,
    #[serde(rename = "first_name")]
    pub employee_fname: Option<String>,
    #[serde(rename = "last_name")]
    pub employee_lname: Option<String>,
    #[serde(rename = "departement")]
    pub employee_departement: Option<String>,
    pub enroll_dow: Option<i64>,
    #[serde(rename = "date")]
    pub enroll_date: Option<i64>,
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
    pub working_time: i64,
    pub late_time: i64,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct ReportsHelper {
    pub employee_id: i32,
    pub employee_fname: Option<String>,
    pub employee_lname: Option<String>,
    pub employee_departement: Option<String>,
    pub device_timezone: i32,
    pub device_name: Option<String>,
    pub device_location: String,
    //pub schedule_name: Option<String>,
    pub enroll_time: i64,
    pub employee_status: i32,
    pub employee_schedule_id: i32,
    pub schedule_type: i32,
}
