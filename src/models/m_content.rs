use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Content {
    pub content_id: i32,
    pub content_title: String,
    pub content_link: String,
    pub content_short: i32,
    pub content_number: i32,
    pub content_sub: i32,
    pub content_parrent: i32,
}
