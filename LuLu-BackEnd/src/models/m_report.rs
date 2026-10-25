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
    #[serde(rename = "department")]
    pub employee_department: Option<String>,
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
    // Seconds worked past the shift end, gated by the `overtime_tolerance_sec` setting.
    // Zero when the employee left on time or within the tolerance window.
    #[serde(rename = "overtime")]
    pub overtime_time: i64,
    /// Name of the holiday when this row was counted on an included holiday day.
    /// Absent (None) on ordinary days, so old clients parsing the JSON ignore it.
    #[serde(rename = "holiday_name", skip_serializing_if = "Option::is_none")]
    pub holiday_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct ReportsHelper {
    pub employee_id: i32,
    pub employee_fname: Option<String>,
    pub employee_lname: Option<String>,
    pub employee_department: Option<String>,
    pub device_timezone: i32,
    pub device_name: Option<String>,
    // Nullable in `devices`: a device that has never been given a location decoded
    // as a hard error, which failed the whole report query and turned EVERY
    // report - including days belonging to correctly configured devices - into a
    // 500. `Reports` already models this column as Option, so match it here.
    pub device_location: Option<String>,
    //pub schedule_name: Option<String>,
    pub enroll_time: i64,
    pub employee_status: i32,
    pub employee_schedule_id: i32,
    pub schedule_type: i32,
}
