use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ReceiverTransaction {
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub user_list: Option<Vec<i32>>,
}