use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ReceiverCMD {
    #[serde(skip_serializing_if = "Option::is_none", rename = "ID")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "Return")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "CMD")]
    pub cmd: Option<String>,
}