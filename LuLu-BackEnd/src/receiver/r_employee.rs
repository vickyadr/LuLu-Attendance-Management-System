use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]

pub struct ReceiverEmployee {
    //#[serde(skip_serializing_if = "Option::is_none", rename = "id")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "first_name")]
    pub fname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "last_name")]
    pub lname: Option<String>,
    //#[serde(skip_serializing_if = "Option::is_none", rename = "address")]
    pub address: Option<String>,
    //#[serde(skip_serializing_if = "Option::is_none", rename = "departement")]
    pub departement: Option<String>,
    //#[serde(skip_serializing_if = "Option::is_none", rename = "status")]
    pub status: Option<i32>,
    //#[serde(skip_serializing_if = "Option::is_none", rename = "schedule_id")]
    pub schedule_id: Option<i32>,
}