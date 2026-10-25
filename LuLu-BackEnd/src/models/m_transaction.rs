use serde::{Deserialize, Serialize};
//use chrono::NaiveDateTime;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct LiveTransaction {
    #[serde(rename = "date_time")]
    pub enroll_time: i64, //NaiveDateTime,
    #[serde(rename = "first_name")]
    pub employee_fname: String,
    #[serde(rename = "last_name")]
    pub employee_lname: String,
    //#[serde(serialize_with = "parse_type")]
    pub enroll_type: i32,
    /// Raw ADMS wire code (OPERLOG operation type or ATTLOG VerifyMode).
    /// NULL when the row predates the column or carried no code. Reported
    /// alongside `enroll_type` so a code LuLu does not map (`enroll_type` 5)
    /// can be identified from data instead of guessed at.
    #[serde(skip_serializing_if = "Option::is_none", rename = "op_code")]
    pub enroll_op_code: Option<i32>,
    #[serde(rename = "location")]
    // Nullable in the schema: a device that has not been given a location
    // yields NULL, and a hard `String` here made every live-transaction decode
    // fail with "unexpected null" -- 500 on the whole feed instead of one row.
    pub device_location: Option<String>,
    #[serde(rename = "tz")]
    pub device_timezone: i32,
    #[serde(rename = "department")]
    pub employee_department: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "status")]
    pub enroll_status: Option<i32>,
    #[serde(skip_deserializing)]
    pub enroll_id: i32,
    pub employee_id: i32,
}
