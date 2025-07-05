use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Command {
    pub command_id: String,
    pub command_name: String,
    pub command_destination: String,
    pub command_params: String,
}
