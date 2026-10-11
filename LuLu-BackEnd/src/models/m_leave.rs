use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct LeaveRequest {
    pub leave_id: i32,
    pub leave_employee_id: i32,
    pub leave_date: chrono::NaiveDate,
    pub leave_kind: String,
    pub leave_reason: String,
    pub leave_status: String,
    pub leave_source: String,
    pub request_by: String,
    pub decided_by: String,
}

#[derive(Deserialize, Debug)]
pub struct ReceiverLeave {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leave_employee_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leave_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leave_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leave_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ReceiverLeaveDecision {
    pub leave_id: i32,
    pub leave_approve: bool,
}

#[derive(Deserialize, Debug)]
pub struct ReceiverLeaveCancel {
    pub leave_id: i32,
}