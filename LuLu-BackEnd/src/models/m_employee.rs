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
    #[serde(rename = "departement")]
    pub employee_departement: Option<String>,
    #[serde(rename = "status")]
    pub employee_status: i32,
    #[serde(rename = "schedule_id")]
    pub employee_schedule_id: Option<i32>,
    //#[serde(skip_serializing_if = "Option::is_none")]
    //pub schedule_name: Option<String>
}