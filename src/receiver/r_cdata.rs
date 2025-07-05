use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GetCData {
    #[serde(skip_serializing_if = "Option::is_none", rename = "SN")]
    pub sn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "INFO")]
    pub info: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ReceiverCData {
    #[serde(skip_serializing_if = "Option::is_none", rename = "SN")]
    pub sn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "Stamp")]
    pub stamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "OpStamp")]
    pub op_stamp: Option<String>,
}