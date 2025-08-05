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
    pub code: i32,
}

#[derive(Serialize)]
pub struct GenericResponse<T> {
    pub message: String,
    pub data: Vec<T>,
    pub code: i32,
}

#[derive(Serialize)]
pub struct KeyValResponse<K, V> {
    pub message: String,
    pub data: HashMap<K, V>,
    pub code: i32,
}

#[derive(Serialize)]
pub struct WildData<K, V> {
    pub wild : K,
    pub value : V,
    pub code: i32,
}

impl NoDataResponse {
    pub fn new(msg: String, e: i32) -> NoDataResponse {
        NoDataResponse {
            message: msg,
            data: Vec::new(),
            code: e,
        }
    }

    pub fn ok(msg: String) -> NoDataResponse {
        NoDataResponse {
            message: msg,
            data: Vec::new(),
            code: 200,
        }
    }
}

impl<T> GenericResponse<T> {
    pub fn new(dt: Vec<T>, msg: String, e: i32) -> GenericResponse<T> {
        GenericResponse::<T> {
            message: msg,
            data: dt,
            code: e,
        }
    }

    pub fn ok(dt: Vec<T>, msg: String) -> GenericResponse<T> {
        GenericResponse::<T> {
            message: msg,
            data: dt,
            code: 200,
        }
    }

    pub fn single(dt: T, msg: String) -> GenericResponse<T> {
        let mut data_single = Vec::<T>::new();
        data_single.insert(0, dt);
        GenericResponse::<T> {
            message: msg,
            data: data_single,
            code: 200,
        }
    }
}

impl<K, V> KeyValResponse<K, V> {
    pub fn new(dt: HashMap<K, V>, msg: String, e: i32) -> KeyValResponse<K, V> {
        KeyValResponse::<K, V> {
            message: msg,
            data: dt,
            code: e,
        }
    }
    
    pub fn ok(dt: HashMap<K, V>, msg: String) -> KeyValResponse<K, V> {
        KeyValResponse::<K, V> {
            message: msg,
            data: dt,
            code: 200,
        }
    }
}
