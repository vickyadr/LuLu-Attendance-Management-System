use std::collections::HashMap;

use serde::Serialize;
use sqlx::PgPool;

pub struct AppState {
    pub db: PgPool,
}
#[derive(Serialize)]
pub struct NoDataResponse {
    pub message: String,
    pub data: Vec<i8>,
}

#[derive(Serialize)]
pub struct GenericResponse<T> {
    pub message: String,
    pub data: Vec<T>,
}

#[derive(Serialize)]
pub struct KeyValResponse<K, V> {
    pub message: String,
    pub data: HashMap<K, V>,
}

impl NoDataResponse {
    pub fn new(msg: String) -> NoDataResponse {
        NoDataResponse {
            message: msg,
            data: Vec::new(),
        }
    }
}

impl<T> GenericResponse<T> {
    pub fn new(dt: Vec<T>, msg: String) -> GenericResponse<T> {
        GenericResponse::<T> {
            message: msg,
            data: dt,
        }
    }
}

impl<K, V> KeyValResponse<K, V> {
    pub fn new(dt: HashMap<K, V>, msg: String) -> KeyValResponse<K, V> {
        KeyValResponse::<K, V> {
            message: msg,
            data: dt,
        }
    }
}
