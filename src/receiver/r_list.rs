use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ReceiverList {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parrent: Option<i32>,
    pub sort: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<i32>,
}
