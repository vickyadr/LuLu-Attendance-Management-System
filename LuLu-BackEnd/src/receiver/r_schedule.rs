//use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ReceiverSchedule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename="shift")]
    pub shift_id: Option<Vec<i32>>,//Option<HashMap<i32, i32>>,
    #[serde(skip_serializing_if = "Option::is_none", rename="pattern")]
    pub pattern: Option<i32>,

}