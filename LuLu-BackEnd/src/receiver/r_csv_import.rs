use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmployeeCsvRow {
    #[serde(rename = "id")]
    pub device_uid: String,
    #[serde(rename = "first_name")]
    pub first_name: String,
    #[serde(rename = "last_name")]
    pub last_name: String,
    #[serde(rename = "departement")]
    pub departement: String,
    #[serde(rename = "address")]
    pub address: String,
    #[serde(rename = "schedule")]
    pub schedule: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ImportError {
    pub row: usize,
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct ImportResult {
    pub inserted: usize,
    pub updated: usize,
    pub errors: Vec<ImportError>,
}