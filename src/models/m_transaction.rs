use serde::{Deserialize, Serialize};
//use chrono::NaiveDateTime;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct LiveTransaction {

    #[serde(rename = "date_time")]
    pub enroll_time: i64,//NaiveDateTime,
    #[serde(rename = "first_name")]
    pub employee_fname: String,
    #[serde(rename = "last_name")]
    pub employee_lname: String,
    //#[serde(serialize_with = "parse_type")]
    pub enroll_type: i32,
    #[serde(rename = "location")]
    pub device_location: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "status")]
    pub enroll_status: Option<i32>,
    #[serde(skip_deserializing)]
    pub enroll_id: i32,
}