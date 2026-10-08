use std::collections::HashMap;

use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;
use sqlx::PgPool;

use crate::utility::wshub::WsHub;

pub struct AppState {
    pub db: PgPool,
    pub ws: WsHub,
}

fn status_for(code: i32) -> StatusCode {
    match code {
        c if (200..300).contains(&c) => StatusCode::OK,
        c if (300..400).contains(&c) => StatusCode::SEE_OTHER,
        400 => StatusCode::BAD_REQUEST,
        401 => StatusCode::UNAUTHORIZED,
        403 => StatusCode::FORBIDDEN,
        404 => StatusCode::NOT_FOUND,
        c if (500..600).contains(&c) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::BAD_REQUEST,
    }
}

pub fn respond<T: Serialize>(code: i32, body: &T) -> HttpResponse {
    HttpResponse::build(status_for(code)).json(body)
}

pub trait IntoResponse {
    fn into_response(self) -> HttpResponse;
}

impl IntoResponse for NoDataResponse {
    fn into_response(self) -> HttpResponse {
        respond(self.code, &self)
    }
}

impl<T: Serialize> IntoResponse for GenericResponse<T> {
    fn into_response(self) -> HttpResponse {
        respond(self.code, &self)
    }
}

impl<K: Serialize, V: Serialize> IntoResponse for KeyValResponse<K, V> {
    fn into_response(self) -> HttpResponse {
        respond(self.code, &self)
    }
}

impl<K: Serialize, V: Serialize> IntoResponse for WildData<K, V> {
    fn into_response(self) -> HttpResponse {
        respond(self.code, &self)
    }
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
    pub wild: K,
    pub value: V,
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
