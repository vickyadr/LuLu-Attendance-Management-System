use serde::Deserialize;
#[derive(Deserialize, Debug)]
pub struct ReceiverDevice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handler: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<i32>,
}
