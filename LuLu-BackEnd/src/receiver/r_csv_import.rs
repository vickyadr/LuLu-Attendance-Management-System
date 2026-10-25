use serde::{Deserialize, Serialize};

// One row of an imported employee CSV. Every field is a String because a CSV
// cell is always text; the handler is the single place that decides what counts
// as a valid number, so a bad cell reports the same way whether it arrived as
// "5" or " 5 " or "five".
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmployeeCsvRow {
    #[serde(rename = "id")]
    pub device_uid: String,
    #[serde(rename = "first_name")]
    pub first_name: String,
    #[serde(rename = "last_name")]
    pub last_name: String,
    #[serde(rename = "department")]
    pub department: String,
    #[serde(rename = "address")]
    pub address: String,
    #[serde(rename = "schedule")]
    pub schedule: String,
}

// A row that could not be imported, with enough detail for the operator to fix
// the file: which line, which column, and what was wrong with it. The import
// continues past these, so one bad row never blocks the whole file.
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