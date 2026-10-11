use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Holiday {
    pub holiday_date: chrono::NaiveDate,
    pub holiday_name: String,
    pub holiday_kind: String,
    pub holiday_source: String,
    #[serde(default)]
    pub holiday_included: bool,
}

#[derive(Deserialize, Debug)]
pub struct ReceiverHoliday {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holiday_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holiday_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holiday_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holiday_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holiday_included: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct ReceiverHolidayImport {
    pub holidays: Vec<HolidayRow>,
}

#[derive(Deserialize, Debug)]
pub struct HolidayRow {
    pub holiday_date: String,
    pub holiday_name: String,
    pub holiday_kind: String,
}
