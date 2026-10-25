use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Employee {
    #[serde(rename = "id")]
    pub employee_id: i32,
    #[serde(rename = "first_name")]
    pub employee_fname: String,
    #[serde(rename = "last_name")]
    pub employee_lname: Option<String>,
    #[serde(rename = "address")]
    pub employee_address: Option<String>,
    #[serde(rename = "department")]
    pub employee_department: Option<String>,
    #[serde(rename = "status")]
    pub employee_status: i32,
    #[serde(rename = "schedule_id")]
    pub employee_schedule_id: Option<i32>,
    /// Device-side user id this employee punches with, or NULL when not linked.
    /// This is the number the machine stores, not the app's employee_id.
    #[serde(rename = "device_uid")]
    pub employee_device_uid: Option<i32>,
    //#[serde(skip_serializing_if = "Option::is_none")]
    //pub schedule_name: Option<String>
}

/// A device-side user id that has punched the machine but has no employee row.
///
/// Punches are stored against the device user id, so an id nobody registered is
/// still recorded. It has no name and no schedule, which is exactly why reports
/// cannot show it and why this listing exists.
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct UnregisteredUser {
    /// Device-side user id the punches carry. Matches `enroll_employee_id`.
    #[serde(rename = "device_uid")]
    pub device_uid: i32,
    /// How many punches this id has produced.
    #[serde(rename = "punch_count")]
    pub punch_count: i64,
    /// Unix time of the earliest punch, for spotting a stale id.
    #[serde(rename = "first_punch")]
    pub first_punch: i64,
    /// Unix time of the latest punch.
    #[serde(rename = "last_punch")]
    pub last_punch: i64,
    /// Timezone offset in hours of the device that owns the newest punch, so
    /// the UI can render the timestamps on that device's clock.
    #[serde(rename = "timezone")]
    pub timezone: Option<i32>,
}
