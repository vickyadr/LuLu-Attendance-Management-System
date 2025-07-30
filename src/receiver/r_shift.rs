use serde::Deserialize;
#[derive(Deserialize, Debug)]
pub struct ReceiverWorkTIme {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_enroll: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_enroll: Option<i64>,
}
